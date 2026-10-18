// SPDX-License-Identifier: GPL-3.0-only
//! Message Receive Thread
//!
//! Authors: MarioS271

use crate::client::state::ClientState;
use crate::client::ui::end_raw_mode;
use crate::encrypt::{EncryptionKey, decrypt};
use crate::packets::framing;
use crate::packets::packet::Packet;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub fn receive_thread(mut read_stream: TcpStream, state_recv: Arc<Mutex<ClientState>>, encryption_key: EncryptionKey) {
    loop {
        match framing::read_message(&mut read_stream) {
            Ok(cipher) => {
                match cipher_to_packet(cipher, encryption_key) {
                    Ok(Packet::Heartbeat) => {},
                    Ok(Packet::Handshake(_)) => {
                        end_raw_mode();
                        eprintln!("Received unexpected handshake packet from server");
                        std::process::exit(1);
                    },
                    Ok(packet @ (Packet::ChatMessage(_) | Packet::SystemMessage(_))) => {
                        state_recv.lock().unwrap().messages.push(packet);
                    },
                    Err(err) => {
                        end_raw_mode();
                        eprintln!("Deserialize Error: {}", err);
                        std::process::exit(1);
                    }
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                end_raw_mode();
                eprintln!("Disconnected");
                std::process::exit(1);
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock
                || err.kind() == std::io::ErrorKind::TimedOut => {
                end_raw_mode();
                eprintln!("Receive Timed Out: {}", err);
                std::process::exit(1);
            }
            Err(err) => {
                end_raw_mode();
                eprintln!("Receive Error: {}", err);
                std::process::exit(1);
            }
        }
    }
}

pub fn cipher_to_packet(cipher: Vec<u8>, encryption_key: EncryptionKey) -> std::io::Result<Packet> {
    let decrypted = match decrypt(encryption_key, cipher.as_slice()) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to decrypt message: {}", e);
            std::process::exit(1);
        }
    };

    Packet::deserialize(decrypted.as_slice())
}

// SPDX-License-Identifier: GPL-3.0-only
//! Per-Client Handler Thread
//!
//! Authors: MarioS271

use crate::client::helpers::sender_name_string_to_bytes;
use crate::encrypt::{EncryptionKey, decrypt};
use crate::packets::message::system_message::SystemMessage;
use crate::packets::packet::Packet;
use crate::server::net::send::send;
use crate::{MAX_MESSAGE_SIZE, PROTOCOL_VERSION};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::broadcast::Sender;
use tokio::sync::broadcast::error::RecvError;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

pub async fn handle_client(
    mut stream: TcpStream,
    broadcast_sender: Sender<Packet>,
    socket_addr: SocketAddr,
    encryption_key: EncryptionKey
) -> std::io::Result<()> {
    let (read_stream, mut write_stream) = stream.split();
    let decoder = LengthDelimitedCodec::builder()
        .max_frame_length(MAX_MESSAGE_SIZE)
        .new_codec();
    let mut framed_reader = FramedRead::new(read_stream, decoder);

    send(Packet::Heartbeat, encryption_key, &mut write_stream).await?;

    let client_name = match framed_reader.next().await {
        Some(Ok(cipher)) => {
            println!(
                "Received {} bytes from an unknown client (this is probably a handshake)",
                cipher.len()
            );

            let bytes = match decrypt(encryption_key, &cipher) {
                Ok(bytes) => bytes,
                Err(err) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        err
                    ));
                }
            };

            let packet = Packet::deserialize(&bytes)?;

            match packet {
                Packet::Handshake(handshake) => {
                    if handshake.protocol_version != PROTOCOL_VERSION {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Client has wrong protocol version (client: {}, server: {})",
                                handshake.protocol_version,
                                PROTOCOL_VERSION
                            )
                        ));
                    }

                    handshake.client_name
                }
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Client sent a non-handshake packet as first packet"
                    ));
                }
            }
        }
        Some(Err(e)) => {
            return Err(std::io::Error::new(
                e.kind(),
                format!("Could not read from socket: {}", e)
            ));
        }
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Unknown client disconnected during handshake phase"
            ));
        }
    };

    let text = format!("{} ({}) connected", client_name, socket_addr);
    println!("{}", text);

    let message = SystemMessage::new(text);
    let _ = broadcast_sender.send(Packet::SystemMessage(message));

    let mut broadcast_receiver = broadcast_sender.subscribe();

    loop {
        tokio::select! {
            socket_received_bytes = framed_reader.next() => {
                match socket_received_bytes {
                    Some(Ok(cipher)) => {
                        println!(
                            "Received {} bytes from {} ({})",
                            cipher.len(),
                            client_name,
                            socket_addr
                        );

                        let bytes = match decrypt(encryption_key, &cipher) {
                            Ok(bytes) => bytes,
                            Err(err) => {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    err
                                ));
                            }
                        };

                        let packet = Packet::deserialize(&bytes)?;

                        match packet {
                            Packet::Heartbeat => {}
                            Packet::ChatMessage(mut chat_msg) => {
                                let client_name_bytes = sender_name_string_to_bytes(client_name.as_str());
                                if chat_msg.sender_name != sender_name_string_to_bytes(client_name.as_str()) {
                                    eprintln!("Warning: client sent a message with a different name, overwriting with initially provided client name");
                                    chat_msg.sender_name = client_name_bytes;
                                }
                                if let Err(e) = broadcast_sender.send(Packet::ChatMessage(chat_msg)) {
                                    return Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        format!("Failed to send message: {}", e)
                                    ));
                                }
                            }
                            Packet::Handshake(_) | Packet::SystemMessage(_) => {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    "Client sent an illegal packet type, kicking client"
                                ));
                            }
                        }
                    }
                    Some(Err(e)) => {
                        return Err(std::io::Error::new(
                            e.kind(),
                            format!("Could not read from socket: {}", e)
                        ));
                    }
                    None => {
                        let text = format!("{} ({}) disconnected", client_name, socket_addr);
                        println!("{}", text);

                        let message = SystemMessage::new(text);
                        let _ = broadcast_sender.send(Packet::SystemMessage(message));

                        return Ok(());
                    }
                }
            }

            broadcast_received_bytes = broadcast_receiver.recv() => {
                match broadcast_received_bytes {
                    Ok(message) => {
                        send(message, encryption_key, &mut write_stream).await?;
                    }
                    Err(RecvError::Lagged(n)) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            format!("Message Overflow; couldn't process {} messages quickly enough", n)
                        ));
                    }
                    Err(RecvError::Closed) => {
                        return Ok(())
                    }
                }
            }
        }
    }
}

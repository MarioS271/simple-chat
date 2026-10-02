// SPDX-License-Identifier: GPL-3.0-only
//! Per-Client Handler Thread
//!
//! Authors: MarioS271

use crate::client::state::EMPTY_SENDER_NAME_ARRAY;
use crate::message::Message;
use crate::{MAX_MESSAGE_SIZE, TCP_TIMEOUT, framing};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::broadcast::Sender;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::timeout;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};
use crate::client::helpers::sender_name_bytes_to_string;

pub async fn handle_client(
    mut stream: TcpStream,
    broadcast_sender: Sender<Message>,
    socket_addr: SocketAddr
) -> std::io::Result<()> {
    stream.write_all(&crate::PROTOCOL_VERSION.to_be_bytes()).await?;

    let mut client_version = [0u8; 2];
    stream.read_exact(&mut client_version).await?;

    if u16::from_be_bytes(client_version) != crate::PROTOCOL_VERSION {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Client has wrong protocol version"
        ));
    }

    let mut client_name = EMPTY_SENDER_NAME_ARRAY;
    stream.read_exact(&mut client_name).await?;

    let mut broadcast_receiver = broadcast_sender.subscribe();

    let (read_stream, mut write_stream) = stream.split();
    let decoder = LengthDelimitedCodec::builder()
        .max_frame_length(MAX_MESSAGE_SIZE)
        .new_codec();
    let mut framed_reader = FramedRead::new(read_stream, decoder);

    println!("{} ({}) connected", sender_name_bytes_to_string(&client_name), socket_addr);

    loop {
        tokio::select! {
            socket_received_bytes = framed_reader.next() => {
                match socket_received_bytes {
                    Some(Ok(bytes)) => {
                        println!(
                            "Received {} bytes from {} ({})",
                            bytes.len(),
                            String::from_utf8_lossy(&client_name),
                            socket_addr
                        );

                        let message = Message::deserialize(&bytes)?;
                        match message {
                            Message::Chat(mut chat_msg) => {
                                if chat_msg.sender_name != client_name {
                                    eprintln!("Warning: client send a message with a different name, overwriting with initially provided client name");
                                    chat_msg.sender_name = client_name;
                                }
                                if let Err(e) = broadcast_sender.send(Message::Chat(chat_msg)) {
                                    return Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        format!("Failed to send message: {}", e)
                                    ));
                                }
                            }
                            Message::System(_) => {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    "Kicking client because it attempted to send a system message"
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
                        println!("{} ({}) disconnected", sender_name_bytes_to_string(&client_name), socket_addr);
                        return Ok(());
                    }
                }
            }

            broadcast_received_bytes = broadcast_receiver.recv() => {
                match broadcast_received_bytes {
                    Ok(message) => {
                        let message_bytes = message.serialize();
                        let mut message_framed = Vec::<u8>::new();
                        framing::write_message(&mut message_framed, message_bytes.as_slice())?;

                        match timeout(TCP_TIMEOUT, write_stream.write_all(message_framed.as_slice())).await {
                            Ok(Ok(())) => {}
                            Ok(Err(e)) => return Err(e),
                            Err(_) => {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::TimedOut,
                                    "Socket Write Timed Out"
                                ));
                            }
                        }
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

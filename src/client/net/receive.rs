// SPDX-License-Identifier: GPL-3.0-only
//! Message Receive Thread
//!
//! Authors: MarioS271

use crate::client::state::ClientState;
use crate::client::ui::end_raw_mode;
use crate::framing;
use crate::message::Message;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub fn receive_thread(mut read_stream: TcpStream, state_recv: Arc<Mutex<ClientState>>) {
    loop {
        match framing::read_message(&mut read_stream) {
            Ok(data) => {
                match Message::deserialize(data.as_slice()) {
                    Ok(msg) => state_recv.lock().unwrap().messages.push(msg),
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
            Err(err) => {
                end_raw_mode();
                eprintln!("Receive Error: {}", err);
                std::process::exit(1);
            }
        }
    }
}

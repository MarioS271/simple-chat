// SPDX-License-Identifier: GPL-3.0-only
//! Common Parsing Data Structures and Logic
//!
//! Authors: MarioS271

use super::client::{ClientCommand, parse_client};
use super::server::{ServerCommand, parse_server};

pub enum Command {
    Client(ClientCommand),
    Server(ServerCommand),
    Version,
    Help
}

pub fn parse(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    match args.next().as_deref() {
        None | Some("--help") => Ok(Command::Help),
        Some("--version") => Ok(Command::Version),

        Some("client") => Ok(Command::Client(parse_client(args)?)),
        Some("server") => Ok(Command::Server(parse_server(args)?)),

        Some(other) => Err(format!("Unknown Option or Subcommand: {}", other))
    }
}

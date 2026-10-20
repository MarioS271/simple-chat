// SPDX-License-Identifier: GPL-3.0-only
//! Program Entrypoint
//!
//! Authors: MarioS271

mod args;
mod client;
mod config;
mod packets;
mod server;

mod encrypt;
mod helpers;

type ProtocolVersion = u16;

const PROTOCOL_VERSION: ProtocolVersion = 1;
const DEFAULT_PORT: u16 = 42003;

const MAX_MESSAGE_SIZE: usize = 4096;    // should stay less than or equal to u16::MAX
const MAX_NAME_LEN: usize = 32;

const TCP_TIMEOUT: std::time::Duration = std::time::Duration::new(30, 0);
const HEARTBEAT_INTERVAL: std::time::Duration = std::time::Duration::new(TCP_TIMEOUT.as_secs() / 3, 0);
const INITIAL_CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::new(5, 0);

fn main() {
    use crate::args::client::ClientCommand;
    use crate::args::common::Command;
    use crate::args::server::ServerCommand;

    let command = args::common::parse(std::env::args().skip(1));

    let run = || -> Result<(), String> {
        match command? {
            Command::Help => {
                println!(indoc::indoc! {r#"
                    Usage: simple-chat <subcommand> [options]

                    Subcommands:
                        client      Connect to or register a server
                        server      Run or manage a server instance

                    Options:
                        --help      Show this message
                        --version   Show this binary's protocol version

                    Run 'simple-chat <subcommand> --help' for
                    subcommand-specific help.
                "#});
                Ok(())
            },
            Command::Version => {
                println!("Protocol Version: {}", PROTOCOL_VERSION);
                Ok(())
            },

            Command::Client(cmd) => match cmd {
                ClientCommand::Connect { target_server_name, user_name } => client::cmd::connect::handler(target_server_name, user_name),
                ClientCommand::Add { name, address } => client::cmd::add::handler(name, address),
                ClientCommand::Remove { name } => client::cmd::remove::handler(name),
                ClientCommand::Help => client::cmd::help::handler(),
                ClientCommand::List => client::cmd::list::handler(),
            }

            Command::Server(cmd) => match cmd {
                ServerCommand::Start { name } => server::cmd::start::handler(name),
                ServerCommand::New { name, port } => server::cmd::new::handler(name, port),
                ServerCommand::Delete { name } => server::cmd::delete::handler(name),
                ServerCommand::ShowKey { name } => server::cmd::show_key::handler(name),
                ServerCommand::Help => server::cmd::help::handler(),
                ServerCommand::List => server::cmd::list::handler(),
            }
        }
    };

    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

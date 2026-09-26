// SPDX-License-Identifier: GPL-3.0-only
//! Program Entrypoint
//!
//! Authors: MarioS271

mod client;
mod config;
mod server;

mod args;
mod encrypt;
mod framing;
mod helpers;
mod message;

pub const PROTOCOL_VERSION: u16 = 1;
pub const DEFAULT_PORT: u16 = 42003;

fn main() {
    use crate::args::common::Command;
    use crate::args::client::ClientCommand;
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
                ClientCommand::Connect { name } => todo!("client connect"),
                ClientCommand::Add { name, address } => client::cmd::add::handler(name, address),
                ClientCommand::Remove { name } => client::cmd::remove::handler(name),
                ClientCommand::Help => client::cmd::help::handler(),
                ClientCommand::List => client::cmd::list::handler(),
            }

            Command::Server(cmd) => match cmd {
                ServerCommand::Start { name } => todo!("server start"),
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

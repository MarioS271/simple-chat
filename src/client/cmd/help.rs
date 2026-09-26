// SPDX-License-Identifier: GPL-3.0-only
//! `client`/`client --help` Command Handler
//!
//! Authors: MarioS271

pub fn handler() -> Result<(), String> {
    println!(indoc::indoc! {r#"
        Usage: simple-chat client <name> [options]

        Arguments:
            name        Name of the server to add, remove or connect to

        Options:
            --add       Add a server with the given name (requires --address)
            --address   The address for the server to add (requires --add, uses port {} if the port is omitted)
            --remove    Remove a saved server with the given name
            --list      Output a list of saved servers
            --help      Show this message
    "#}, crate::DEFAULT_PORT);
    Ok(())
}

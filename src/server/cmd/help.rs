// SPDX-License-Identifier: GPL-3.0-only
//! `server`/`server --help` Command Handler
//!
//! Authors: MarioS271

pub fn handler() -> Result<(), String> {
    println!(indoc::indoc! {r#"
        Usage: simple-chat server <name> [options]

        Arguments:
            name        Name of the server to create, delete or start

        Options:
            --new       Add a server with the given name
            --delete    Delete an existing server which has the given name
            --show-key  Print the given server's encryption key as text
                        and as a QR code
            --list      Output a list of created servers
            --help      Show this message
    "#});
    Ok(())
}

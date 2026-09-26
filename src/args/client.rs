// SPDX-License-Identifier: GPL-3.0-only
//! Client Parsing Data Structures and Logic
//!
//! Authors: MarioS271

pub enum ClientCommand {
    Connect { name: String },
    Add { name: String, port: Option<u16> },
    Remove { name: String },
    List,
    Help
}

pub fn parse_client(args: impl Iterator<Item = String>) -> Result<ClientCommand, String> {
    let mut name: String = String::new();
    let mut port: Option<u16> = None;

    let mut mode = ClientMode::Default;

    for arg in args {
        match arg.as_str() {
            "--help" => return Ok(ClientCommand::Help),
            "--list" => return Ok(ClientCommand::List),
            "--add" => {
                if mode == ClientMode::Default {
                    mode = ClientMode::Add;
                } else {
                    return Err(format!("Cannot use --add with {}", mode.to_flag_str()));
                }
            },
            "--remove" => {
                if mode == ClientMode::Default {
                    mode = ClientMode::Remove;
                } else {
                    return Err(format!("Cannot use --remove with {}", mode.to_flag_str()));
                }
            },
            other if other.starts_with("--") => return Err(format!("Unknown Option: {}", other)),
            other => name = other.to_string()
        }
    }

    if name.is_empty() && mode == ClientMode::Default {
        return Ok(ClientCommand::Help);
    }

    if name.is_empty() {
        return Err("Missing 'name' argument".to_string());
    }

    if mode == ClientMode::Add {
        return Ok(ClientCommand::Add { name, port })
    }
    if mode == ClientMode::Remove {
        return Ok(ClientCommand::Remove { name })
    }

    Ok(ClientCommand::Connect { name })
}

#[derive(PartialEq, Eq)]
enum ClientMode {
    Default,
    Add,
    Remove
}
impl ClientMode {
    pub fn to_flag_str(&self) -> String {
        use ClientMode::*;
        match self {
            Add => "--add",
            Remove => "--remove",
            _ => ""
        }.to_string()
    }
}

// SPDX-License-Identifier: GPL-3.0-only
//! Server Parsing Data Structures and Logic
//!
//! Authors: MarioS271

pub enum ServerCommand {
    Start { name: String },
    New { name: String, port: Option<u16> },
    Delete { name: String },
    ShowKey { name: String },
    List,
    Help
}

pub fn parse_server(mut args: impl Iterator<Item = String>) -> Result<ServerCommand, String> {
    let mut name: String = String::new();
    let mut port: Option<u16> = None;

    let mut mode = ServerMode::Default;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => return Ok(ServerCommand::Help),
            "--list" => return Ok(ServerCommand::List),
            "--new" => {
                if mode != ServerMode::Default {
                    mode = ServerMode::New
                } else {
                    return Err(format!("Cannot use --new with {}", mode.to_flag_str()));
                }
            },
            "--delete" => {
                if mode != ServerMode::Default {
                    mode = ServerMode::Delete
                } else {
                    return Err(format!("Cannot use --delete with {}", mode.to_flag_str()));
                }
            }
            "--show-key" => {
                if mode != ServerMode::Default {
                    mode = ServerMode::ShowKey
                } else {
                    return Err(format!("Cannot use --show-key with {}", mode.to_flag_str()));
                }
            },
            "--port" => {
                let value = args.next().ok_or("--port requires a value".to_string())?;
                port = Some(value.parse::<u16>().map_err(|_| format!("Invalid Port: {}", value))?);
            }
            other if other.starts_with("--") => return Err(format!("Unknown Option: {}", other)),
            other => name = other.to_string()
        }
    }

    if name.is_empty() && mode == ServerMode::Default {
        return Ok(ServerCommand::Help);
    }

    if name.is_empty() {
        return Err("Missing 'name' argument".to_string());
    }

    if mode == ServerMode::New {
        return Ok(ServerCommand::New { name, port })
    }
    if mode == ServerMode::Delete {
        return Ok(ServerCommand::Delete { name })
    }
    if mode == ServerMode::ShowKey {
        return Ok(ServerCommand::ShowKey { name })
    }

    Ok(ServerCommand::Start { name })
}

#[derive(PartialEq, Eq)]
enum ServerMode {
    Default,
    New,
    Delete,
    ShowKey
}
impl ServerMode {
    pub fn to_flag_str(&self) -> String {
        use ServerMode::*;
        match self {
            New => "--new",
            Delete => "--delete",
            ShowKey => "--show-key",
            _ => ""
        }.to_string()
    }
}

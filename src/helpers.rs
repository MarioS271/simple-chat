// SPDX-License-Identifier: GPL-3.0-only
//! Helpers such as [`get_timestamp`]
//!
//! Authors: MarioS271

use std::io::Write;

pub fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn ask_for_input(prompt: &str) -> std::io::Result<String> {
    print!("{}\n> ", prompt);
    std::io::stdout().flush()?;

    let stdin = std::io::stdin();
    let mut result = String::new();

    stdin.read_line(&mut result)?;

    if result.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No input received"
        ));
    }

    Ok(result.trim().to_string())
}

pub fn ask_for_confirmation(prompt: &str, default_is_yes: bool) -> std::io::Result<bool> {
    print!(
        "{} {} ",
        prompt,
        match default_is_yes {
            true => "(Y/n)",
            false => "(y/N)"
        }
    );
    std::io::stdout().flush()?;

    let mut result = String::new();
    std::io::stdin().read_line(&mut result)?;

    Ok(matches!(result.trim(), "y" | "Y"))
}

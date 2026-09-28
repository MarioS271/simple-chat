// SPDX-License-Identifier: GPL-3.0-only
//! `server --show-key` Command Handler
//!
//! Authors: MarioS271

use qrcode::QrCode;
use qrcode::render::unicode;

pub fn handler(name: String) -> Result<(), String> {
    let config = crate::config::server::load()?;
    let server = config.servers().iter()
        .find(|server| server.name == name)
        .ok_or(format!("Server '{}' not found", name))?;

    let qr_code = QrCode::new(server.key.as_bytes())
        .map_err(|e| format!("Could not generate QR code: {}", e))?;
    let qr_image = qr_code.render::<unicode::Dense1x2>().build();

    println!("Key of Server '{}': {}", server.name, server.key);
    println!("{}", qr_image);

    Ok(())
}

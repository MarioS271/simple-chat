// SPDX-License-Identifier: GPL-3.0-only
//! Server-Side Logic
//!
//! Authors: MarioS271

pub(crate) mod cmd;
pub(crate) mod handle_client;

use std::time::Duration;
pub const xREAD_TIMEOUT: Duration = Duration::new(30, 0);

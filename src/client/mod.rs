// SPDX-License-Identifier: GPL-3.0-only
//! Client-Side Logic
//!
//! Authors: MarioS271

pub(crate) mod cmd;
pub(crate) mod net;
pub(crate) mod helpers;
pub(crate) mod state;
pub(crate) mod ui;

use std::time::Duration;
pub const READ_TIMEOUT: Duration = Duration::new(30, 0);
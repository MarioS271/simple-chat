// SPDX-License-Identifier: GPL-3.0-only
//! The formatted trait to output cleanly formatted timestamps and content strings
//!
//! Authors: MarioS271

use std::borrow::Cow;

pub trait Formatted {
    fn __timestamp(&self) -> u64;
    fn __content(&self) -> &[u8];

    fn formatted_timestamp(&self) -> String {
        let datetime = chrono::DateTime::from_timestamp(self.__timestamp() as i64, 0);

        if let Some(datetime) = datetime {
            datetime
                .with_timezone(&chrono::Local)
                .format("%H:%M:%S")
                .to_string()
        } else {
            "??:??:??".to_string()
        }
    }

    fn formatted_content(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.__content())
    }
}

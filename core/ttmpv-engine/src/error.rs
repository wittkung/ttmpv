// SPDX-License-Identifier: BSD-3-Clause OR Apache-2.0
//
// Copyright (c) 2026 Witt Kung <witt.w.kung@gmail.com>
// All rights reserved.
//
// TTMPV: High-performance native media engine and headless playback core.

use std::ffi::CStr;
use thiserror::Error;
use crate::ffi;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MpvError {
    #[error("Failed to allocate mpv handle")]
    AllocationFailed,

    #[error("Failed to initialize mpv handle: {0}")]
    InitializationFailed(String),

    #[error("MPV internal error code {code}: {message}")]
    Internal {
        code: i32,
        message: String,
    },

    #[error("Invalid C-string argument: {0}")]
    InvalidCString(String),

    #[error("Property error: {0}")]
    PropertyError(String),

    #[error("Engine channel disconnected")]
    ChannelDisconnected,
}

impl MpvError {
    pub fn from_code(code: i32) -> Result<(), Self> {
        if code >= 0 {
            Ok(())
        } else {
            let msg = unsafe {
                let ptr = ffi::mpv_error_string(code);
                if ptr.is_null() {
                    "Unknown error".to_string()
                } else {
                    CStr::from_ptr(ptr).to_string_lossy().into_owned()
                }
            };
            Err(Self::Internal { code, message: msg })
        }
    }
}

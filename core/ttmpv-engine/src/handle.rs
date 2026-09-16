// SPDX-License-Identifier: BSD-3-Clause OR Apache-2.0
//
// Copyright (c) 2026 Witt Kung <witt.w.kung@gmail.com>
// All rights reserved.
//
// TTMPV: High-performance native media engine and headless playback core.

use std::ffi::{CStr, CString};
use std::ptr;
use crate::error::MpvError;
use crate::events::Event;
use crate::ffi;

/// Specifies the precision and strategy for seek operations.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SeekMode {
    /// Jumps to the exact requested frame or timestamp by decoding intermediate frames.
    Exact,
    /// Jumps to the nearest keyframe (fastest, lower precision).
    Keyframe,
    /// Seeks relative to current position.
    Relative,
}

impl SeekMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Keyframe => "keyframes",
            Self::Relative => "relative",
        }
    }
}

/// RAII wrapper for an mpv client context.
pub struct MpvHandle {
    ptr: *mut ffi::MpvHandleOpaque,
}

unsafe impl Send for MpvHandle {}
unsafe impl Sync for MpvHandle {}

impl MpvHandle {
    /// Allocates and initializes an mpv handle tailored for headless and embedded operation.
    pub fn new() -> Result<Self, MpvError> {
        let handle = unsafe { ffi::mpv_create() };
        if handle.is_null() {
            return Err(MpvError::AllocationFailed);
        }

        let mut instance = Self { ptr: handle };

        // Configure default sandboxed headless parameters
        instance.set_option_string("force-window", "no")?;
        instance.set_option_string("osc", "no")?;
        instance.set_option_string("osd-bar", "no")?;
        instance.set_option_string("load-scripts", "no")?;
        instance.set_option_string("config", "no")?;

        let init_status = unsafe { ffi::mpv_initialize(instance.ptr) };
        if init_status < 0 {
            return Err(MpvError::InitializationFailed(format!(
                "mpv_initialize failed with status {}",
                init_status
            )));
        }

        Ok(instance)
    }

    /// Returns the raw underlying pointer.
    pub fn as_raw(&self) -> *mut ffi::MpvHandleOpaque {
        self.ptr
    }

    /// Sets an option on the mpv instance before or after initialization.
    pub fn set_option_string(&mut self, name: &str, value: &str) -> Result<(), MpvError> {
        let c_name = CString::new(name).map_err(|e| MpvError::InvalidCString(e.to_string()))?;
        let c_val = CString::new(value).map_err(|e| MpvError::InvalidCString(e.to_string()))?;

        let res = unsafe {
            ffi::mpv_set_option_string(self.ptr, c_name.as_ptr(), c_val.as_ptr())
        };
        MpvError::from_code(res)
    }

    /// Sets a string property dynamically.
    pub fn set_property_string(&self, name: &str, value: &str) -> Result<(), MpvError> {
        let c_name = CString::new(name).map_err(|e| MpvError::InvalidCString(e.to_string()))?;
        let c_val = CString::new(value).map_err(|e| MpvError::InvalidCString(e.to_string()))?;

        let res = unsafe {
            ffi::mpv_set_property_string(self.ptr, c_name.as_ptr(), c_val.as_ptr())
        };
        MpvError::from_code(res)
    }

    /// Gets a string property dynamically.
    pub fn get_property_string(&self, name: &str) -> Result<Option<String>, MpvError> {
        let c_name = CString::new(name).map_err(|e| MpvError::InvalidCString(e.to_string()))?;
        let ptr = unsafe { ffi::mpv_get_property_string(self.ptr, c_name.as_ptr()) };

        if ptr.is_null() {
            Ok(None)
        } else {
            let s = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
            unsafe { ffi::mpv_free(ptr as *mut std::os::raw::c_void) };
            Ok(Some(s))
        }
    }

    /// Sends a high-level command array to mpv.
    pub fn command(&self, args: &[&str]) -> Result<(), MpvError> {
        let mut c_args: Vec<CString> = Vec::with_capacity(args.len());
        for arg in args {
            c_args.push(CString::new(*arg).map_err(|e| MpvError::InvalidCString(e.to_string()))?);
        }

        let mut pt_args: Vec<*const std::os::raw::c_char> = c_args.iter().map(|s| s.as_ptr()).collect();
        pt_args.push(ptr::null());

        let res = unsafe { ffi::mpv_command(self.ptr, pt_args.as_mut_ptr()) };
        MpvError::from_code(res)
    }

    /// Performs a high-precision seek to the given timestamp in seconds.
    pub fn seek(&self, seconds: f64, mode: SeekMode) -> Result<(), MpvError> {
        let sec_str = format!("{:.4}", seconds);
        self.command(&["seek", &sec_str, "absolute", mode.as_str()])
    }

    /// Convenience method for frame-accurate exact seek tailored for subtitle synchronization.
    pub fn seek_exact(&self, seconds: f64) -> Result<(), MpvError> {
        self.seek(seconds, SeekMode::Exact)
    }

    /// Observes a property change with a registered userdata tag.
    pub fn observe_property(&self, id: u64, name: &str, format: ffi::MpvFormat) -> Result<(), MpvError> {
        let c_name = CString::new(name).map_err(|e| MpvError::InvalidCString(e.to_string()))?;
        let res = unsafe {
            ffi::mpv_observe_property(self.ptr, id, c_name.as_ptr(), format)
        };
        MpvError::from_code(res)
    }

    /// Blocks for up to `timeout` seconds waiting for the next mpv event.
    pub fn wait_event(&self, timeout: f64) -> Option<Event> {
        let raw_ev = unsafe { ffi::mpv_wait_event(self.ptr, timeout) };
        unsafe { Event::from_raw(raw_ev) }
    }

    /// Returns the upstream mpv client API version.
    pub fn api_version() -> (u16, u16) {
        let ver = unsafe { ffi::mpv_client_api_version() };
        let major = (ver >> 16) as u16;
        let minor = (ver & 0xFFFF) as u16;
        (major, minor)
    }
}

impl Drop for MpvHandle {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ffi::mpv_destroy(self.ptr);
            }
            self.ptr = ptr::null_mut();
        }
    }
}

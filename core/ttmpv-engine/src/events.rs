// SPDX-License-Identifier: BSD-3-Clause OR Apache-2.0
//
// Copyright (c) 2026 Witt Kung <witt.w.kung@gmail.com>
// All rights reserved.
//
// TTMPV: High-performance native media engine and headless playback core.

use std::ffi::CStr;
use crate::ffi;

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    String(String),
    Flag(bool),
    Int64(i64),
    Double(f64),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Shutdown,
    StartFile,
    FileLoaded,
    EndFile {
        reason: i32,
        error: i32,
    },
    Seek,
    PlaybackRestart,
    PropertyChange {
        id: u64,
        name: String,
        value: PropertyValue,
    },
    QueueOverflow,
    Other(i32),
}

impl Event {
    pub(crate) unsafe fn from_raw(raw: *mut ffi::MpvEvent) -> Option<Self> {
        if raw.is_null() {
            return None;
        }

        let ev = &*raw;
        match ev.event_id {
            ffi::MpvEventId::None => None,
            ffi::MpvEventId::Shutdown => Some(Event::Shutdown),
            ffi::MpvEventId::StartFile => Some(Event::StartFile),
            ffi::MpvEventId::FileLoaded => Some(Event::FileLoaded),
            ffi::MpvEventId::EndFile => {
                if !ev.data.is_null() {
                    let end_data = &*(ev.data as *const ffi::MpvEventEndFile);
                    Some(Event::EndFile {
                        reason: end_data.reason,
                        error: end_data.error,
                    })
                } else {
                    Some(Event::EndFile {
                        reason: 0,
                        error: ev.error,
                    })
                }
            }
            ffi::MpvEventId::Seek => Some(Event::Seek),
            ffi::MpvEventId::PlaybackRestart => Some(Event::PlaybackRestart),
            ffi::MpvEventId::PropertyChange => {
                if !ev.data.is_null() {
                    let prop = &*(ev.data as *const ffi::MpvEventProperty);
                    let name = if prop.name.is_null() {
                        String::new()
                    } else {
                        CStr::from_ptr(prop.name).to_string_lossy().into_owned()
                    };

                    let value = if prop.data.is_null() {
                        PropertyValue::None
                    } else {
                        match prop.format {
                            ffi::MpvFormat::String | ffi::MpvFormat::OsdString => {
                                let cstr_ptr = *(prop.data as *const *const std::os::raw::c_char);
                                if cstr_ptr.is_null() {
                                    PropertyValue::None
                                } else {
                                    PropertyValue::String(
                                        CStr::from_ptr(cstr_ptr).to_string_lossy().into_owned(),
                                    )
                                }
                            }
                            ffi::MpvFormat::Flag => {
                                let flag = *(prop.data as *const std::os::raw::c_int);
                                PropertyValue::Flag(flag != 0)
                            }
                            ffi::MpvFormat::Int64 => {
                                let val = *(prop.data as *const i64);
                                PropertyValue::Int64(val)
                            }
                            ffi::MpvFormat::Double => {
                                let val = *(prop.data as *const f64);
                                PropertyValue::Double(val)
                            }
                            _ => PropertyValue::None,
                        }
                    };

                    Some(Event::PropertyChange {
                        id: ev.reply_userdata,
                        name,
                        value,
                    })
                } else {
                    None
                }
            }
            ffi::MpvEventId::QueueOverflow => Some(Event::QueueOverflow),
            _ => Some(Event::Other(ev.event_id as i32)),
        }
    }
}

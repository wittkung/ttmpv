// SPDX-License-Identifier: BSD-3-Clause OR Apache-2.0
//
// Copyright (c) 2026 Witt Kung <witt.w.kung@gmail.com>
// All rights reserved.
//
// TTMPV: High-performance native media engine and headless playback core.

use std::os::raw::{c_char, c_double, c_int, c_longlong, c_void};

pub enum MpvHandleOpaque {}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MpvFormat {
    None = 0,
    String = 1,
    OsdString = 2,
    Flag = 3,
    Int64 = 4,
    Double = 5,
    Node = 6,
    NodeArray = 7,
    NodeMap = 8,
    ByteArray = 9,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MpvEventId {
    None = 0,
    Shutdown = 1,
    LogMessage = 2,
    GetPropertyReply = 3,
    SetPropertyReply = 4,
    CommandReply = 5,
    StartFile = 6,
    EndFile = 7,
    FileLoaded = 8,
    ClientMessage = 16,
    VideoReconfig = 17,
    AudioReconfig = 18,
    Seek = 20,
    PlaybackRestart = 21,
    PropertyChange = 22,
    QueueOverflow = 24,
    Hook = 25,
}

#[repr(C)]
pub struct MpvEvent {
    pub event_id: MpvEventId,
    pub error: c_int,
    pub reply_userdata: u64,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct MpvEventProperty {
    pub name: *const c_char,
    pub format: MpvFormat,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct MpvEventEndFile {
    pub reason: c_int,
    pub error: c_int,
    pub playlist_entry_id: c_longlong,
    pub playlist_insert_id: c_longlong,
    pub playlist_insert_num_entries: c_int,
}

extern "C" {
    pub fn mpv_client_api_version() -> std::os::raw::c_ulong;
    pub fn mpv_error_string(error: c_int) -> *const c_char;
    pub fn mpv_free(data: *mut c_void);

    pub fn mpv_create() -> *mut MpvHandleOpaque;
    pub fn mpv_initialize(ctx: *mut MpvHandleOpaque) -> c_int;
    pub fn mpv_destroy(ctx: *mut MpvHandleOpaque);
    pub fn mpv_terminate_destroy(ctx: *mut MpvHandleOpaque);

    pub fn mpv_command(ctx: *mut MpvHandleOpaque, args: *mut *const c_char) -> c_int;
    pub fn mpv_command_string(ctx: *mut MpvHandleOpaque, args: *const c_char) -> c_int;

    pub fn mpv_set_option_string(
        ctx: *mut MpvHandleOpaque,
        name: *const c_char,
        data: *const c_char,
    ) -> c_int;

    pub fn mpv_set_property(
        ctx: *mut MpvHandleOpaque,
        name: *const c_char,
        format: MpvFormat,
        data: *mut c_void,
    ) -> c_int;

    pub fn mpv_set_property_string(
        ctx: *mut MpvHandleOpaque,
        name: *const c_char,
        data: *const c_char,
    ) -> c_int;

    pub fn mpv_get_property(
        ctx: *mut MpvHandleOpaque,
        name: *const c_char,
        format: MpvFormat,
        data: *mut c_void,
    ) -> c_int;

    pub fn mpv_get_property_string(
        ctx: *mut MpvHandleOpaque,
        name: *const c_char,
    ) -> *mut c_char;

    pub fn mpv_observe_property(
        ctx: *mut MpvHandleOpaque,
        reply_userdata: u64,
        name: *const c_char,
        format: MpvFormat,
    ) -> c_int;

    pub fn mpv_unobserve_property(
        ctx: *mut MpvHandleOpaque,
        registered_reply_userdata: u64,
    ) -> c_int;

    pub fn mpv_wait_event(ctx: *mut MpvHandleOpaque, timeout: c_double) -> *mut MpvEvent;
    pub fn mpv_wakeup(ctx: *mut MpvHandleOpaque);
    pub fn mpv_set_wakeup_callback(
        ctx: *mut MpvHandleOpaque,
        cb: Option<extern "C" fn(ctx: *mut c_void)>,
        d: *mut c_void,
    );
}

// SPDX-License-Identifier: BSD-3-Clause OR Apache-2.0
//
// Copyright (c) 2026 Witt Kung <witt.w.kung@gmail.com>
// All rights reserved.
//
// TTMPV: High-performance native media engine and headless playback core.

import Foundation
import Observation
import CTTMpvBridge
import os.log

/// Safe trampoline to route mpv wakeup callbacks back to the player instance.
private final class WakeupBox: @unchecked Sendable {
    private let lock = NSLock()
    private var callback: (@Sendable () -> Void)?

    init(callback: (@Sendable () -> Void)?) {
        self.callback = callback
    }

    func fire() {
        lock.lock()
        let cb = callback
        lock.unlock()
        cb?()
    }

    func invalidate() {
        lock.lock()
        callback = nil
        lock.unlock()
    }
}

private func globalWakeupCallback(ctx: UnsafeMutableRawPointer?) {
    guard let ctx else { return }
    let box = Unmanaged<WakeupBox>.fromOpaque(ctx).takeUnretainedValue()
    box.fire()
}

/// Modern Swift 6 `@Observable` media player facade driven by libmpv.
@Observable
public final class MPVPlayer: @unchecked Sendable {
    private let logger = Logger(subsystem: "com.metastudyline.ttmpv", category: "MPVPlayer")
    private let lock = NSLock()
    
    public private(set) var isPlaying: Bool = false
    public private(set) var duration: Double = 0.0
    public private(set) var position: Double = 0.0
    public private(set) var mediaTitle: String = ""

    private var handle: OpaquePointer?
    private var wakeupBox: WakeupBox?
    private var eventQueue = DispatchQueue(label: "com.metastudyline.ttmpv.eventQueue", qos: .userInitiated)

    public init() {
        initializeEngine()
    }

    deinit {
        destroyEngine()
    }

    private func initializeEngine() {
        guard let ctx = mpv_create() else {
            logger.error("Failed to allocate mpv handle")
            return
        }

        mpv_set_option_string(ctx, "force-window", "no")
        mpv_set_option_string(ctx, "osc", "no")
        mpv_set_option_string(ctx, "osd-bar", "no")
        mpv_set_option_string(ctx, "load-scripts", "no")
        mpv_set_option_string(ctx, "config", "no")

        let status = mpv_initialize(ctx)
        guard status >= 0 else {
            logger.error("Failed to initialize mpv handle: \(status)")
            mpv_destroy(ctx)
            return
        }

        self.handle = ctx

        let box = WakeupBox { [weak self] in
            self?.processPendingEvents()
        }
        self.wakeupBox = box

        let unmanaged = Unmanaged.passUnretained(box).toOpaque()
        mpv_set_wakeup_callback(ctx, globalWakeupCallback, unmanaged)
    }

    private func destroyEngine() {
        lock.lock()
        wakeupBox?.invalidate()
        wakeupBox = nil
        let ptr = handle
        handle = nil
        lock.unlock()

        if let ptr {
            mpv_destroy(ptr)
        }
    }

    /// Loads a local or remote media URL into the playback engine.
    public func loadFile(url: URL) {
        command(["loadfile", url.path])
        self.isPlaying = true
    }

    /// Toggles playback pause state.
    public func togglePause() {
        command(["cycle", "pause"])
    }

    /// Seeks playback to absolute or relative offset in seconds.
    public func seek(to seconds: Double, exact: Bool = false) {
        let mode = exact ? "exact" : "keyframes"
        command(["seek", String(format: "%.3f", seconds), "absolute", mode])
    }

    /// Sets playback audio volume level [0.0, 100.0].
    public func setVolume(_ volume: Double) {
        let clamped = max(0.0, min(100.0, volume))
        setPropertyString(name: "volume", value: String(format: "%.1f", clamped))
    }

    /// Selects active audio track by track ID.
    public func setAudioTrack(id: Int) {
        setPropertyString(name: "aid", value: String(id))
    }

    /// Selects active subtitle track by track ID.
    public func setSubtitleTrack(id: Int) {
        setPropertyString(name: "sid", value: String(id))
    }

    /// Sets an arbitrary property string on the player instance.
    public func setPropertyString(name: String, value: String) {
        lock.lock()
        guard let ctx = handle else {
            lock.unlock()
            return
        }
        lock.unlock()

        mpv_set_property_string(ctx, (name as NSString).utf8String, (value as NSString).utf8String)
    }

    /// Dispatches a high-level command string array to mpv.
    public func command(_ args: [String]) {
        lock.lock()
        guard let ctx = handle else {
            lock.unlock()
            return
        }
        lock.unlock()

        var cStrings: [UnsafePointer<CChar>?] = args.map { ($0 as NSString).utf8String }
        cStrings.append(nil)

        _ = cStrings.withUnsafeMutableBufferPointer { buf in
            mpv_command(ctx, buf.baseAddress)
        }
    }

    private func processPendingEvents() {
        eventQueue.async { [weak self] in
            guard let self else { return }
            while true {
                self.lock.lock()
                guard let ctx = self.handle else {
                    self.lock.unlock()
                    break
                }
                self.lock.unlock()

                let eventPtr = mpv_wait_event(ctx, 0.0)
                guard let ev = eventPtr, ev.pointee.event_id != MPV_EVENT_NONE else {
                    break
                }

                if ev.pointee.event_id == MPV_EVENT_SHUTDOWN {
                    break
                }
            }
        }
    }

    /// Returns the active mpv client API version.
    public static var apiVersion: (major: Int, minor: Int) {
        let ver = mpv_client_api_version()
        return (Int(ver >> 16), Int(ver & 0xFFFF))
    }
}

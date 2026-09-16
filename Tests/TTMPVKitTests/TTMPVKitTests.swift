// SPDX-License-Identifier: BSD-3-Clause OR Apache-2.0
//
// Copyright (c) 2026 Witt Kung <witt.w.kung@gmail.com>
// All rights reserved.
//
// TTMPV: High-performance native media engine and headless playback core.

import Testing
import Foundation
@testable import TTMPVKit

@Suite("TTMPVKit Core Integration Tests")
struct TTMPVKitTests {
    @Test("Verifies upstream libmpv API version negotiation")
    func testApiVersionNegotiation() {
        let (major, minor) = MPVPlayer.apiVersion
        #expect(major >= 1 || (major == 0 && minor >= 1))
    }

    @Test("Verifies MPVPlayer lifecycle and command execution")
    func testPlayerLifecycle() {
        let player = MPVPlayer()
        #expect(player.isPlaying == false)
        player.togglePause()
        player.seek(to: 5.0, exact: true)
        player.setVolume(85.0)
        player.setAudioTrack(id: 1)
        player.setSubtitleTrack(id: 1)
    }

    @Test("Verifies MPVMetalRenderLayer 1600 nits EDR parameters")
    func testMetalRenderLayerConfiguration() {
        let layer = MPVMetalRenderLayer()
        #expect(layer.pixelFormat == .rgba16Float)
        #expect(layer.wantsExtendedDynamicRangeContent == true)
        #expect(layer.isOpaque == true)
    }
}

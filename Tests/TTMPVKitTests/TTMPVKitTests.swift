// SPDX-License-Identifier: BSD-3-Clause OR Apache-2.0
//
// Copyright (c) 2026 Witt Kung <witt.w.kung@gmail.com>
// All rights reserved.
//
// TTMPV: High-performance native media engine and headless playback core.

import XCTest
import Foundation
@testable import TTMPVKit

final class TTMPVKitTests: XCTestCase {
    func testApiVersionNegotiation() {
        let (major, minor) = MPVPlayer.apiVersion
        XCTAssertTrue(major >= 1 || (major == 0 && minor >= 1))
    }

    func testPlayerLifecycle() {
        let player = MPVPlayer()
        XCTAssertFalse(player.isPlaying)
        player.togglePause()
        player.seek(to: 5.0, exact: true)
        player.setVolume(85.0)
        player.setAudioTrack(id: 1)
        player.setSubtitleTrack(id: 1)
    }

    func testMetalRenderLayerConfiguration() {
        let layer = MPVMetalRenderLayer()
        XCTAssertEqual(layer.pixelFormat, .rgba16Float)
        XCTAssertTrue(layer.wantsExtendedDynamicRangeContent)
        XCTAssertTrue(layer.isOpaque)
    }
}

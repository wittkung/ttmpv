// SPDX-License-Identifier: BSD-3-Clause OR Apache-2.0
//
// Copyright (c) 2026 Witt Kung <witt.w.kung@gmail.com>
// All rights reserved.
//
// TTMPV: High-performance native media engine and headless playback core.

import AppKit
import QuartzCore
import Metal
import CTTMpvBridge
import os.log

/// Thread-safe weak proxy enabling Sendable closure invocation without retaining CALayers.
private final class MPVMetalLayerProxy: @unchecked Sendable {
    weak var layer: MPVMetalRenderLayer?
    init(layer: MPVMetalRenderLayer) { self.layer = layer }
    func trigger() {
        layer?.requestRender()
    }
}

/// Pure Metal hardware passthrough layer unlocking Apple 1600 nits Liquid Retina XDR EDR headroom.
///
/// Configures a 16-bit floating point (`.rgba16Float`) texture pipeline mapped into extended linear sRGB
/// color space, bypassing standard 8-bit SDR clamp boundaries and driving direct zero-copy frame presentation.
public final class MPVMetalRenderLayer: CAMetalLayer, @unchecked Sendable {
    private let logger = Logger(subsystem: "com.metastudyline.ttmpv", category: "MPVMetalRenderLayer")
    private let renderQueue = DispatchQueue(label: "com.metastudyline.ttmpv.metalRenderQueue", qos: .userInteractive)
    
    private var proxy: MPVMetalLayerProxy?
    private var commandQueue: (any MTLCommandQueue)?
    public var onRenderRequest: (@Sendable () -> Void)?

    public override init() {
        super.init()
        configureEDRMetalPipeline()
    }

    public override init(layer: Any) {
        super.init(layer: layer)
        configureEDRMetalPipeline()
    }

    public required init?(coder: NSCoder) {
        super.init(coder: coder)
        configureEDRMetalPipeline()
    }

    /// Configures 1600 nits EDR CAMetalLayer parameters.
    private func configureEDRMetalPipeline() {
        self.device = MTLCreateSystemDefaultDevice()
        if let dev = self.device {
            self.commandQueue = dev.makeCommandQueue()
        }

        self.proxy = MPVMetalLayerProxy(layer: self)

        // 1600 nits EDR Hardware Passthrough Configuration
        self.pixelFormat = .rgba16Float
        self.colorspace = CGColorSpace(name: CGColorSpace.extendedLinearSRGB)
        self.wantsExtendedDynamicRangeContent = true
        self.isOpaque = true
        self.framebufferOnly = false
        self.allowsNextDrawableTimeout = false
        self.needsDisplayOnBoundsChange = true
        self.autoresizingMask = [.layerWidthSizable, .layerHeightSizable]
        self.contentsGravity = .resizeAspect
    }

    /// Triggers an asynchronous redraw request on the render queue.
    public func requestRender() {
        renderQueue.async { [weak self] in
            guard let self else { return }
            self.onRenderRequest?()
        }
    }
}

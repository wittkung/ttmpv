// swift-tools-version: 6.0
// SPDX-License-Identifier: BSD-3-Clause OR Apache-2.0
//
// Copyright (c) 2026 Witt Kung <witt.w.kung@gmail.com>
// All rights reserved.
//
// TTMPV: High-performance native media engine and headless playback core.

import PackageDescription

let swiftSettings: [SwiftSetting] = [
    .enableUpcomingFeature("ExistentialAny"),
    .enableExperimentalFeature("StrictConcurrency")
]

let package = Package(
    name: "TTMPVKit",
    platforms: [
        .macOS(.v14),
        .iOS(.v17)
    ],
    products: [
        .library(name: "TTMPVKit", targets: ["TTMPVKit", "CTTMpvBridge"])
    ],
    targets: [
        .target(
            name: "CTTMpvBridge",
            path: "Sources/CTTMpvBridge",
            publicHeadersPath: "include",
            cSettings: [
                .headerSearchPath("include"),
                .define("GL_SILENCE_DEPRECATION")
            ],
            linkerSettings: [
                .linkedLibrary("mpv"),
                .unsafeFlags([
                    "-L/opt/homebrew/lib",
                    "-L/usr/local/lib",
                    "-Xlinker", "-rpath",
                    "-Xlinker", "/opt/homebrew/lib"
                ])
            ]
        ),
        .target(
            name: "TTMPVKit",
            dependencies: [
                "CTTMpvBridge"
            ],
            path: "Sources/TTMPVKit",
            swiftSettings: swiftSettings
        ),
        .testTarget(
            name: "TTMPVKitTests",
            dependencies: [
                "TTMPVKit"
            ],
            path: "Tests/TTMPVKitTests",
            swiftSettings: swiftSettings
        )
    ]
)

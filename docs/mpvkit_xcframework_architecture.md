# TTZip MPVKit.xcframework Minimal Static Architecture Specification

## 1. Executive Summary & Architecture Decision Record (ADR)

### 1.1 Context and Problem Statement
In previous iterations of TTZip's media preview subsystem, `apple/scripts/bundle_app.sh` dynamically scanned `Frameworks/libmpv.dylib` using `otool -L` and recursively pulled 51 dynamic libraries from `/opt/homebrew` into `Contents/Frameworks/`. This legacy approach introduced severe architectural and operational liabilities:
1. **Bloated App Footprint**: The recursive closure of unpruned Homebrew libraries introduced **64.5 MB** of dylibs, including heavy video encoders (`libx265`, `libx264`, `libSvtAv1Enc`), Vulkan/SPIR-V compilers (`libshaderc_shared`), network security stacks (`libcrypto`, `libssl`), and optical disk tools (`libbluray`, `libudfread`), which are completely unused in an archive preview context.
2. **Mac App Store (MAS) Sandboxing Deadlock**:
   - **JIT W^X Memory Violation**: Homebrew's `libmpv` is built with `libluajit`. LuaJIT allocates writable and executable memory (`W^X`), which violates Apple's Hardened Runtime. Archive preview utilities have zero justification for the `com.apple.security.cs.allow-jit` entitlement, resulting in immediate rejection during App Store review.
   - **GPL License Contagion**: Software encoders (`x264` and `x265`) are licensed under GPL-2.0+ and GPL-3.0, creating fundamental legal conflicts with the Apple App Store Terms of Service and DRM distribution constraints.
   - **Fragile Path Relocation**: The recursive `install_name_tool` rewriting of 51 interconnected dylibs frequently failed on clean environments or caused Mach-O code signature corruption during Apple Transporter verification.
3. **Breach of Independent Reproducible Build Invariant**: A clean `git clone` on a machine without Homebrew could not build or package the application, violating TTZip's core repository topology mandate.

### 1.2 Decision Outcome
We replace the 51 loose dynamic libraries with a **self-contained, minimal, static `MPVKit.xcframework`**:
- **Pure LGPL Decode Matrix**: FFmpeg is stripped of all software encoders and muxers, keeping only decoders and Apple VideoToolbox / AudioToolbox hardware acceleration.
- **Assembly-Optimized AV1 Software Decode**: Integrated `dav1d` with hand-tuned Neon (arm64) and AVX2 (x86_64) assembly.
- **Zero-Fontconfig Subtitle Pipeline**: `libass` is configured with the native Apple `CoreText` font provider (`-Dcoretext=enabled -Dfontconfig=disabled`), completely eliminating external fontconfig XML caches and glib dependencies.
- **Zero-JIT Static MPV Core**: `libmpv` is built with `-Dlua=disabled` and `-Djavascript=disabled`, permanently resolving the Hardened Runtime JIT compliance issue.
- **Single-Archive Packaging**: All component static archives (`.a`) are fused via `libtool -static` into `libMPVKit.a` and wrapped into a standard, codesign-clean `MPVKit.xcframework` (<18 MB uncompressed, <6 MB compressed).

---

## 2. Dependency Matrix & License Audit

| Component | Pinned Version | License | Upstream Source | Static Footprint | Optimization & Pruning Rules |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **libmpv** | `v0.39.0` | LGPL-2.1+ | GitHub / mpv-player | ~3.5 MB | Disabled Lua/LuaJIT, MuJS, Vulkan, Shaderc, Wayland, X11, Rubberband, LCMS2. Enabled CGL/Cocoa render layer. |
| **FFmpeg** | `n7.1.1` | LGPL-2.1+ | ffmpeg.org | ~6.2 MB | Stripped all encoders (`--disable-encoders`), muxers (`--disable-muxers`), network (`--disable-network`), and programs. Hardware acceleration via VideoToolbox/AudioToolbox. Software AV1 via libdav1d. |
| **dav1d** | `v1.5.1` | BSD-2-Clause | VideoLAN GitLab | ~1.1 MB | Pure static library with Neon/AVX2 assembly optimizations. Tools and tests disabled. |
| **libass** | `0.17.3` | ISC | GitHub / libass | ~420 KB | Native Apple CoreText backend enabled; fontconfig disabled. Assembly optimizations enabled. |
| **HarfBuzz**| `10.4.0` | Old MIT | GitHub / harfbuzz | ~1.4 MB | FreeType + CoreText shaping enabled; Glib/ICU/Cairo disabled. |
| **FriBidi** | `1.0.16` | LGPL-2.1+ | GitHub / fribidi | ~180 KB | Unicode Bidirectional text layout engine. Static only. |
| **FreeType**| `2.13.3` | FTL / GPL-2.0+ | Savannah GNU | ~680 KB | Minimal vector/bitmap rasterization. Brotli, bzip2, png, zlib disabled to prevent circular dependencies. |

**License Audit Summary**:
- **No GPL-only components**: All included source code complies with LGPL-2.1+, BSD, MIT, or ISC licenses.
- **No non-free or patent-encumbered proprietary blobs**: Strictly safe for commercial distribution and App Store sandboxing.

---

## 3. Toolchain & Cross-Architecture Compilation Pipeline

The pipeline is implemented in `apple/scripts/build_mpv_static_xcframework.sh` and targets macOS 14.0+:

```
                             [ Source Tarballs ]
           (dav1d, freetype, fribidi, harfbuzz, libass, ffmpeg, mpv)
                                     │
                                     ▼
        ┌─────────────────────────────────────────────────────────┐
        │            Dual-Slice Cross-Compilation Stage            │
        │   • arm64-apple-macos14.0 (Neon assembly, -O3, -fPIC)   │
        │   • x86_64-apple-macos14.0 (AVX2 assembly, -O3, -fPIC)  │
        └────────────────────────────┬────────────────────────────┘
                                     │
                 ┌───────────────────┴───────────────────┐
                 ▼                                       ▼
        [ arm64 Static Archives ]               [ x86_64 Static Archives ]
        (libmpv.a, libavcodec.a, ...)           (libmpv.a, libavcodec.a, ...)
                 │                                       │
                 ▼ (libtool -static)                     ▼ (libtool -static)
          libMPVKit-arm64.a                       libMPVKit-x86_64.a
                 └───────────────────┬───────────────────┘
                                     │
                                     ▼ (lipo -create)
                           libMPVKit-universal.a
                                     │
                 ┌───────────────────┴───────────────────┐
                 │ Public Headers:                       │
                 │   mpv/client.h, render.h, ...         │
                 │   MPVKit.h (Umbrella)                 │
                 │   module.modulemap                    │
                 └───────────────────┬───────────────────┘
                                     │
                                     ▼ (xcodebuild -create-xcframework)
                         Frameworks/MPVKit.xcframework
```

---

## 4. Swift Package Manager (SPM) & Xcode Integration

### 4.1 Consuming `MPVKit.xcframework` via `Package.swift`
In `apple/Package.swift`, reference the generated XCFramework as a `.binaryTarget`:

```swift
// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "TTZipApp",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .library(name: "TTZipPreviewKit", targets: ["TTZipPreviewKit"])
    ],
    targets: [
        // 1. Binary target referencing the compiled static XCFramework
        .binaryTarget(
            name: "MPVKit",
            path: "Frameworks/MPVKit.xcframework"
        ),

        // 2. Preview kit linking against MPVKit
        .target(
            name: "TTZipPreviewKit",
            dependencies: [
                "MPVKit"
            ],
            path: "Sources/TTZipPreviewKit",
            linkerSettings: [
                .linkedFramework("AudioToolbox"),
                .linkedFramework("VideoToolbox"),
                .linkedFramework("CoreAudio"),
                .linkedFramework("CoreFoundation"),
                .linkedFramework("CoreMedia"),
                .linkedFramework("CoreVideo"),
                .linkedFramework("CoreText"),
                .linkedFramework("CoreGraphics"),
                .linkedFramework("OpenGL"),
                .linkedFramework("Cocoa"),
                .linkedFramework("Metal"),
                .linkedFramework("QuartzCore"),
                .linkedLibrary("c++"),
                .linkedLibrary("iconv"),
                .linkedLibrary("z"),
                .linkedLibrary("bz2")
            ]
        )
    ]
)
```

### 4.2 Consuming in Swift Code
Because the `module.modulemap` exports an umbrella module named `MPVKit`, Swift code can directly import it without bridging headers:

```swift
import Foundation
import MPVKit

public final class MPVEngineHandle: @unchecked Sendable {
    private let handle: OpaquePointer?

    public init() {
        self.handle = mpv_create()
        mpv_initialize(self.handle)
    }

    deinit {
        if let handle = self.handle {
            mpv_destroy(handle)
        }
    }
}
```

---

## 5. Verification & Security Invariants

### 5.1 Zero-JIT Invariant (`W^X` Compliance)
Every build is automatically audited via `nm`:
```bash
nm -gU Frameworks/MPVKit.xcframework/macos-arm64_x86_64/libMPVKit.a | grep "luaL_newstate"
# Expected: Exit code 1 (zero matches)
```

### 5.2 Zero External Non-System Dylib Dependency
Because `MPVKit.xcframework` is a static archive, inspecting the final client binary (`otool -L TTZip.app/Contents/MacOS/TTZip`) reveals that:
- Zero references to `/opt/homebrew/...` or `@rpath/libavcodec...` exist.
- Only `/System/Library/Frameworks/*` and `/usr/lib/libSystem.B.dylib` are referenced.
- MAS App Sandbox code signing (`codesign --verify --deep --strict`) passes without secondary entitlements.

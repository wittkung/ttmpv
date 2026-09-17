# ttmpv

High-Performance Cross-Platform Native Media Engine & Headless Streaming Core.

## Architecture

`ttmpv` is decoupled into three orthogonal layers:
- `sys/`: Hermetic, sandboxed C compilation matrix producing minimal, static `libmpv.a` and `MPVKit.xcframework` stripped of JIT interpreters and software encoders.
- `core/`: Pure Rust micro-kernel (`ttmpv-engine`) providing RAII handle safety, thread-isolated event loops, lock-free state channels, and direct PCM/waveform stream taps for audio/subtitle pipelines.
- `apple/`: Apple Silicon native rendering facade (`TTMPVKit`) unlocking 1600 nits Liquid Retina XDR EDR headroom via pure `CAMetalLayer`.

## Prerequisites & Dependencies

To build and test `ttmpv` locally:

### 1. Standalone Swift Package Manager (SPM) Builds
When building or testing standalone with SwiftPM (`swift build`, `swift test`), the host environment must provide upstream `libmpv` C headers and runtime shared libraries:
- **macOS (Homebrew)**:
  ```bash
  brew install mpv
  ```
  `Package.swift` automatically links against `-lmpv` with library search paths configured for `/opt/homebrew/lib` (Apple Silicon) and `/usr/local/lib` (Intel x86_64).
- **Custom / Linux Environments**:
  Ensure `mpv/client.h` and `mpv/render.h` are located in your standard C include path (`/usr/include` or `CPATH`) and `libmpv.so` / `libmpv.dylib` is accessible in the library linker search path (`LIBRARY_PATH`).

### 2. Monorepo Bazel Builds
Hermetic builds in the workspace resolve dependencies via `//third_party/mpv:mpv`:
```bash
# Build all targets in products/ttmpv
bazel build //products/ttmpv/...

# Run unit and integration tests
bazel test //products/ttmpv:TTMPVKitTests
```

### 3. Static Production XCFramework (`MPVKit.xcframework`)
For App Store Sandboxing and Zero-JIT hardened runtime compliance without dynamic Homebrew dylibs, compile the minimal static framework via:
```bash
./sys/scripts/build_mpv_static_xcframework.sh
```

## Testing

Verify the test suite across both build systems:

```bash
# Bazel test execution (XCTest discovery bridge)
bazel test //products/ttmpv:TTMPVKitTests --test_output=all

# Swift Package Manager native execution
swift test --package-path products/ttmpv
```

## License

- Core & Facades: `BSD-3-Clause OR Apache-2.0`
- C Components: Compliant with `LGPL-2.1-or-later`

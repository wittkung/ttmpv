# ttmpv

High-Performance Cross-Platform Native Media Engine & Headless Streaming Core.

## Architecture

`ttmpv` is decoupled into three orthogonal layers:
- `sys/`: Hermetic, sandboxed C compilation matrix producing minimal, static `libmpv.a` and `MPVKit.xcframework` stripped of JIT interpreters and software encoders.
- `core/`: Pure Rust micro-kernel (`ttmpv-engine`) providing RAII handle safety, thread-isolated event loops, lock-free state channels, and direct PCM/waveform stream taps for audio/subtitle pipelines.
- `apple/`: Apple Silicon native rendering facade (`TTMPVKit`) unlocking 1600 nits Liquid Retina XDR EDR headroom via pure `CAMetalLayer`.

## License

- Core & Facades: `BSD-3-Clause OR Apache-2.0`
- C Components: Compliant with `LGPL-2.1-or-later`

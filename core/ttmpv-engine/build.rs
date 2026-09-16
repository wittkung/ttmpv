// SPDX-License-Identifier: BSD-3-Clause OR Apache-2.0
//
// Copyright (c) 2026 Witt Kung <witt.w.kung@gmail.com>
// All rights reserved.
//
// TTMPV: High-performance native media engine and headless playback core.

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // 1. Probe via standard pkg-config
    if pkg_config::probe_library("mpv").is_ok() {
        return;
    }

    // 2. Fallback search paths for macOS / Homebrew
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" {
        let homebrew_lib = PathBuf::from("/opt/homebrew/lib");
        if homebrew_lib.join("libmpv.dylib").exists() || homebrew_lib.join("libmpv.a").exists() {
            println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
            println!("cargo:rustc-link-lib=mpv");
            return;
        }

        let intel_lib = PathBuf::from("/usr/local/lib");
        if intel_lib.join("libmpv.dylib").exists() || intel_lib.join("libmpv.a").exists() {
            println!("cargo:rustc-link-search=native=/usr/local/lib");
            println!("cargo:rustc-link-lib=mpv");
            return;
        }
    }

    // 3. Default fallback
    println!("cargo:rustc-link-lib=mpv");
}

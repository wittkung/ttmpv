// SPDX-License-Identifier: BSD-3-Clause OR Apache-2.0
//
// Copyright (c) 2026 Witt Kung <witt.w.kung@gmail.com>
// All rights reserved.
//
// TTMPV: High-performance native media engine and headless playback core.

pub mod ffi;
pub mod error;
pub mod events;
pub mod handle;
pub mod audio;

pub use error::MpvError;
pub use events::{Event, PropertyValue};
pub use handle::{MpvHandle, SeekMode};
pub use audio::{AudioConfig, AudioExtractor, PcmChunk, WaveformOverview};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpv_api_version() {
        let (major, minor) = MpvHandle::api_version();
        println!("Loaded mpv client API version: {}.{}", major, minor);
        assert!(major >= 1 || (major == 0 && minor >= 1));
    }

    #[test]
    fn test_mpv_handle_lifecycle() {
        let handle = MpvHandle::new().expect("Failed to initialize mpv instance");
        assert!(!handle.as_raw().is_null());

        // Verify property querying
        let pause_val = handle.get_property_string("pause").expect("Failed to get pause property");
        assert!(pause_val.is_some());
    }

    #[test]
    fn test_pcm_waveform_calculation() {
        let samples = vec![0.0, 0.5, -0.5, 1.0, -1.0, 0.2, -0.2, 0.8];
        let chunk = PcmChunk {
            pts: 0.0,
            sample_rate: 48000,
            channels: 1,
            samples,
        };

        let peaks = chunk.compute_waveform_peaks(4);
        assert_eq!(peaks.len(), 4);
        for peak in peaks {
            assert!((0.0..=1.0).contains(&peak));
        }

        let overview = chunk.generate_overview();
        assert!(overview.duration > 0.0);
    }

    #[test]
    fn test_headless_audio_extractor_setup() {
        let config = AudioConfig::whisper();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channels, 1);

        let handle = AudioExtractor::create_headless_extractor(&config)
            .expect("Failed to initialize headless audio extractor");
        assert!(!handle.as_raw().is_null());
    }

    #[test]
    fn test_seek_exact_dispatch() {
        let handle = MpvHandle::new().expect("Failed to create handle");
        // When no file is loaded, mpv returns an error or rejects the command
        let res = handle.seek_exact(10.5);
        println!("Seek on empty player returned: {:?}", res);
        assert!(res.is_err());
    }
}

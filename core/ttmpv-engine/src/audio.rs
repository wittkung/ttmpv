// SPDX-License-Identifier: BSD-3-Clause OR Apache-2.0
//
// Copyright (c) 2026 Witt Kung <witt.w.kung@gmail.com>
// All rights reserved.
//
// TTMPV: High-performance native media engine and headless playback core.

use std::path::Path;
use crate::error::MpvError;
use crate::handle::MpvHandle;

/// Audio decoding and extraction configuration tailored for subtitle and ASR engines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioConfig {
    /// Target sample rate in Hz (standard Whisper default: 16000).
    pub sample_rate: u32,
    /// Number of channels (standard Whisper default: 1 for mono).
    pub channels: u16,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
        }
    }
}

impl AudioConfig {
    /// Constructs a standard Whisper-compliant 16kHz mono configuration.
    pub fn whisper() -> Self {
        Self::default()
    }

    /// Constructs high-fidelity stereo audio configuration.
    pub fn hi_fi() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
        }
    }
}

/// Streamed linear PCM audio buffer block tailored for ASR (Whisper/VAD) and waveform rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct PcmChunk {
    /// Presentation timestamp (PTS) in seconds.
    pub pts: f64,
    /// Sample rate in Hertz (e.g. 16000, 44100, 48000).
    pub sample_rate: u32,
    /// Number of interleaved audio channels.
    pub channels: u16,
    /// Interleaved normalized 32-bit floating point audio samples [-1.0, 1.0].
    pub samples: Vec<f32>,
}

impl PcmChunk {
    /// Computes downsampled Root-Mean-Square (RMS) peak waveform envelope bins.
    pub fn compute_waveform_peaks(&self, bins: usize) -> Vec<f32> {
        if bins == 0 || self.samples.is_empty() {
            return Vec::new();
        }

        let total_frames = self.samples.len() / (self.channels as usize).max(1);
        let frames_per_bin = (total_frames / bins).max(1);
        let mut peaks = Vec::with_capacity(bins);

        for bin_idx in 0..bins {
            let start_frame = bin_idx * frames_per_bin;
            let end_frame = (start_frame + frames_per_bin).min(total_frames);
            if start_frame >= total_frames {
                break;
            }

            let mut sum_sq = 0.0f32;
            let mut count = 0;
            for f in start_frame..end_frame {
                let sample_idx = f * (self.channels as usize);
                if sample_idx < self.samples.len() {
                    let s = self.samples[sample_idx];
                    sum_sq += s * s;
                    count += 1;
                }
            }

            let rms = if count > 0 { (sum_sq / count as f32).sqrt() } else { 0.0 };
            peaks.push(rms.min(1.0));
        }

        peaks
    }

    /// Generates a multi-resolution pyramid overview for rapid canvas rendering.
    pub fn generate_overview(&self) -> WaveformOverview {
        let total_seconds = self.samples.len() as f64 / (self.sample_rate as f64 * self.channels as f64);
        
        let coarse_bins = (total_seconds * 10.0).ceil() as usize;
        let medium_bins = (total_seconds * 50.0).ceil() as usize;
        let detailed_bins = (total_seconds * 200.0).ceil() as usize;

        WaveformOverview {
            duration: total_seconds,
            coarse_10hz: self.compute_waveform_peaks(coarse_bins),
            medium_50hz: self.compute_waveform_peaks(medium_bins),
            detailed_200hz: self.compute_waveform_peaks(detailed_bins),
        }
    }
}

/// Multi-tier resolution waveform pyramid for sub-millisecond timeline canvas zoom levels.
#[derive(Debug, Clone, PartialEq)]
pub struct WaveformOverview {
    pub duration: f64,
    /// 10 samples per second overview.
    pub coarse_10hz: Vec<f32>,
    /// 50 samples per second editing view.
    pub medium_50hz: Vec<f32>,
    /// 200 samples per second micro-timing alignment view.
    pub detailed_200hz: Vec<f32>,
}

/// Headless batch audio extractor for lightning-fast offline transcode and ASR ingestion.
pub struct AudioExtractor;

impl AudioExtractor {
    /// Creates and configures a headless, clock-unshackled mpv instance for batch PCM dumping.
    pub fn create_headless_extractor(config: &AudioConfig) -> Result<MpvHandle, MpvError> {
        let mut handle = MpvHandle::new()?;

        // Unshackle playback clock to decode at maximum CPU throughput
        handle.set_option_string("untimed", "yes")?;
        handle.set_option_string("speed", "100.0")?;
        handle.set_option_string("video", "no")?;
        handle.set_option_string("sub", "no")?;

        // Configure audio resampler filter
        let format_filter = format!(
            "lavfi=[aresample={},pan=mono|c0=0.5*c0+0.5*c1]",
            config.sample_rate
        );
        if config.channels == 1 {
            let _ = handle.set_option_string("af", &format_filter);
        }

        Ok(handle)
    }

    /// Verifies that a target audio file exists and can be probed by the engine.
    pub fn probe_duration(path: &Path) -> Result<f64, MpvError> {
        let handle = Self::create_headless_extractor(&AudioConfig::default())?;
        let path_str = path.to_str().ok_or_else(|| {
            MpvError::InvalidCString("Path contains invalid UTF-8 characters".to_string())
        })?;

        handle.command(&["loadfile", path_str, "replace"])?;

        // Read duration property
        if let Some(dur_str) = handle.get_property_string("duration")? {
            if let Ok(dur) = dur_str.parse::<f64>() {
                return Ok(dur);
            }
        }

        Ok(0.0)
    }
}

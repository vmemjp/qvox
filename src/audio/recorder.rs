use std::io::Cursor;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result, bail};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Maximum recording duration in seconds.
const MAX_RECORDING_SECS: u32 = 60;

/// Recording state exposed to the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingState {
    Idle,
    Recording,
}

/// Microphone recorder using cpal.
///
/// `cpal::Stream` is `!Send`, so `Recorder` must live on the thread where
/// it was created (typically the main/UI thread).
pub struct Recorder {
    stream: Option<cpal::Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
    state: RecordingState,
}

impl std::fmt::Debug for Recorder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Recorder")
            .field("state", &self.state)
            .field("sample_rate", &self.sample_rate)
            .field("channels", &self.channels)
            .finish_non_exhaustive()
    }
}

impl Recorder {
    /// Create a new recorder using the default input device.
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("no input device available")?;

        let supported = device
            .default_input_config()
            .context("no default input config")?;

        Ok(Self {
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: supported.sample_rate().0,
            channels: supported.channels(),
            state: RecordingState::Idle,
        })
    }

    /// Current recording state.
    pub fn state(&self) -> RecordingState {
        self.state
    }

    /// Sample rate of the input device.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Start recording from the microphone.
    pub fn start(&mut self) -> Result<()> {
        if self.state == RecordingState::Recording {
            bail!("already recording");
        }

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("no input device available")?;

        let supported = device
            .default_input_config()
            .context("no default input config")?;

        self.sample_rate = supported.sample_rate().0;
        self.channels = supported.channels();

        let max_samples = self.sample_rate as usize * MAX_RECORDING_SECS as usize;
        let channels = self.channels;

        // Clear buffer
        if let Ok(mut buf) = self.buffer.lock() {
            buf.clear();
        }

        let buffer = Arc::clone(&self.buffer);
        let config: cpal::StreamConfig = supported.into();

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut buf) = buffer.try_lock() {
                        // Stop collecting after max duration
                        if buf.len() >= max_samples {
                            return;
                        }

                        // Convert to mono: take first channel only
                        if channels > 1 {
                            for chunk in data.chunks(channels as usize) {
                                buf.push(chunk[0]);
                            }
                        } else {
                            buf.extend_from_slice(data);
                        }
                    }
                },
                |err| {
                    eprintln!("recording stream error: {err}");
                },
                None,
            )
            .context("failed to build input stream")?;

        stream.play().context("failed to start recording")?;
        self.stream = Some(stream);
        self.state = RecordingState::Recording;

        Ok(())
    }

    /// Stop recording and return the captured mono f32 samples.
    pub fn stop(&mut self) -> Vec<f32> {
        // Drop the stream to stop recording
        self.stream.take();
        self.state = RecordingState::Idle;

        if let Ok(mut buf) = self.buffer.lock() {
            std::mem::take(&mut *buf)
        } else {
            Vec::new()
        }
    }

    /// Get the current number of recorded samples (for elapsed time display).
    pub fn recorded_samples(&self) -> usize {
        self.buffer
            .lock()
            .map(|buf| buf.len())
            .unwrap_or(0)
    }

    /// Get recorded duration in seconds.
    #[allow(clippy::cast_precision_loss)]
    pub fn elapsed_secs(&self) -> f32 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        self.recorded_samples() as f32 / self.sample_rate as f32
    }
}

/// Encode mono f32 samples as WAV bytes (16-bit PCM).
#[allow(clippy::cast_possible_truncation)]
pub fn samples_to_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer =
            hound::WavWriter::new(&mut cursor, spec).context("failed to create WAV writer")?;
        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let int_sample = (clamped * 32767.0) as i16;
            writer
                .write_sample(int_sample)
                .context("failed to write WAV sample")?;
        }
        writer.finalize().context("failed to finalize WAV")?;
    }

    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_to_wav_round_trip() {
        let samples = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        let wav_bytes = samples_to_wav(&samples, 16_000).expect("encode");

        // Verify it's valid WAV
        let cursor = Cursor::new(wav_bytes);
        let mut reader = hound::WavReader::new(cursor).expect("decode");
        let spec = reader.spec();
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.sample_rate, 16_000);
        assert_eq!(spec.bits_per_sample, 16);

        let decoded: Vec<i16> = reader.samples::<i16>().map(|s| s.expect("sample")).collect();
        assert_eq!(decoded.len(), 5);

        // Check approximate values (16-bit quantization)
        assert_eq!(decoded[0], 0);
        assert!((decoded[1] - 16383).abs() <= 1);
        assert!((decoded[2] + 16383).abs() <= 1);
        assert_eq!(decoded[3], 32767);
        assert_eq!(decoded[4], -32767);
    }

    #[test]
    fn samples_to_wav_empty() {
        let wav_bytes = samples_to_wav(&[], 44_100).expect("encode empty");
        let cursor = Cursor::new(wav_bytes);
        let reader = hound::WavReader::new(cursor).expect("decode");
        assert_eq!(reader.len(), 0);
    }

    #[test]
    fn samples_to_wav_clamps_values() {
        let samples = vec![2.0, -3.0]; // out of range
        let wav_bytes = samples_to_wav(&samples, 16_000).expect("encode");
        let cursor = Cursor::new(wav_bytes);
        let mut reader = hound::WavReader::new(cursor).expect("decode");
        let decoded: Vec<i16> = reader.samples::<i16>().map(|s| s.expect("sample")).collect();
        assert_eq!(decoded[0], 32767); // clamped to 1.0
        assert_eq!(decoded[1], -32767); // clamped to -1.0
    }

    #[test]
    fn recording_state_default() {
        // Just test the enum
        assert_eq!(RecordingState::Idle, RecordingState::Idle);
        assert_ne!(RecordingState::Idle, RecordingState::Recording);
    }
}

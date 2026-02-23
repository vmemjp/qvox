use std::io::Cursor;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use futures_util::StreamExt;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Default model file name.
const MODEL_FILENAME: &str = "ggml-base.bin";

/// `HuggingFace` URL for the default Whisper model.
const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin";

/// Target sample rate for Whisper input.
const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Return the directory where Whisper models are stored.
///
/// Path: `{data_dir}/qvox/models/`
pub fn models_dir() -> Result<PathBuf> {
    let data = dirs::data_dir().context("could not determine data directory")?;
    Ok(data.join("qvox").join("models"))
}

/// Return the full path to the default Whisper model.
pub fn default_model_path() -> Result<PathBuf> {
    Ok(models_dir()?.join(MODEL_FILENAME))
}

/// Check whether the default model is already downloaded.
pub fn model_exists() -> bool {
    default_model_path().is_ok_and(|p| p.exists())
}

/// Download the Whisper model from `HuggingFace`.
///
/// Calls `on_progress(bytes_downloaded, total_bytes)` periodically.
/// `total_bytes` may be 0 if the server does not provide `Content-Length`.
pub async fn download_model<F>(on_progress: F) -> Result<PathBuf>
where
    F: Fn(u64, u64),
{
    use tokio::io::AsyncWriteExt;

    let model_path = default_model_path()?;

    if model_path.exists() {
        return Ok(model_path);
    }

    let dir = model_path.parent().context("invalid model path")?;
    tokio::fs::create_dir_all(dir)
        .await
        .context("failed to create models directory")?;

    let response = reqwest::get(MODEL_URL)
        .await
        .context("failed to start model download")?
        .error_for_status()
        .context("model download returned error status")?;

    let total = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let tmp_path = model_path.with_extension("bin.tmp");
    let mut file = tokio::fs::File::create(&tmp_path)
        .await
        .context("failed to create temp model file")?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("error reading download stream")?;
        file.write_all(&chunk)
            .await
            .context("failed to write model chunk")?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total);
    }

    file.flush().await.context("failed to flush model file")?;
    drop(file);

    tokio::fs::rename(&tmp_path, &model_path)
        .await
        .context("failed to rename temp model file")?;

    Ok(model_path)
}

/// Load a WAV file (any sample rate, any bit depth) and return mono f32
/// samples resampled to 16 kHz.
pub fn load_wav_16khz_mono(wav_bytes: &[u8]) -> Result<Vec<f32>> {
    let cursor = Cursor::new(wav_bytes);
    let mut reader = hound::WavReader::new(cursor).context("failed to read WAV header")?;
    let spec = reader.spec();

    // Read samples as f32
    let raw_samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| s.context("failed to read float sample"))
            .collect::<Result<Vec<_>>>()?,
        hound::SampleFormat::Int => {
            let bits = spec.bits_per_sample;
            #[allow(clippy::cast_precision_loss)]
            let max_val = (1_i64 << (bits - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| {
                    #[allow(clippy::cast_precision_loss)]
                    let v = s.context("failed to read int sample")? as f32 / max_val;
                    Ok(v)
                })
                .collect::<Result<Vec<_>>>()?
        }
    };

    // Convert to mono if stereo
    let mono = if spec.channels > 1 {
        raw_samples
            .chunks(usize::from(spec.channels))
            .map(|frame| frame.iter().sum::<f32>() / f32::from(spec.channels))
            .collect()
    } else {
        raw_samples
    };

    // Resample to 16 kHz if needed
    if spec.sample_rate == TARGET_SAMPLE_RATE {
        return Ok(mono);
    }

    Ok(linear_resample(&mono, spec.sample_rate, TARGET_SAMPLE_RATE))
}

/// Simple linear interpolation resampler.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn linear_resample(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if input.is_empty() || from_rate == to_rate {
        return input.to_vec();
    }

    let ratio = f64::from(from_rate) / f64::from(to_rate);
    let out_len = ((input.len() as f64) / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = (src_pos - idx as f64) as f32;

        let sample = if idx + 1 < input.len() {
            input[idx] * (1.0 - frac) + input[idx + 1] * frac
        } else {
            input[input.len() - 1]
        };
        output.push(sample);
    }

    output
}

/// Transcribe audio bytes (WAV format) using the default Whisper model.
///
/// This is a blocking operation and should be called via
/// `tokio::task::spawn_blocking`.
pub fn transcribe(wav_bytes: &[u8]) -> Result<String> {
    let model_path = default_model_path()?;
    if !model_path.exists() {
        bail!("Whisper model not found at {}", model_path.display());
    }

    let audio = load_wav_16khz_mono(wav_bytes)?;
    transcribe_with_model(&model_path, &audio)
}

/// Transcribe pre-processed 16 kHz mono f32 audio using the model at
/// the given path.
fn transcribe_with_model(model_path: &Path, audio: &[f32]) -> Result<String> {
    let ctx = WhisperContext::new_with_params(
        &model_path.to_string_lossy(),
        WhisperContextParameters::default(),
    )
    .map_err(|e| anyhow::anyhow!("failed to load Whisper model: {e}"))?;

    let mut state = ctx
        .create_state()
        .map_err(|e| anyhow::anyhow!("failed to create Whisper state: {e}"))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });
    params.set_n_threads(num_threads());
    params.set_language(None); // auto-detect
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    state
        .full(params, audio)
        .map_err(|e| anyhow::anyhow!("Whisper transcription failed: {e}"))?;

    let mut text = String::new();
    for segment in state.as_iter() {
        if let Ok(s) = segment.to_str() {
            text.push_str(s);
        }
    }

    Ok(text.trim().to_owned())
}

/// Return the cache path for a given audio file hash.
///
/// The cache file is stored at `{data_dir}/qvox/cache/{hash}.txt`.
pub fn cache_path(audio_hash: &str) -> Result<PathBuf> {
    let data = dirs::data_dir().context("could not determine data directory")?;
    Ok(data.join("qvox").join("cache").join(format!("{audio_hash}.txt")))
}

/// Look up cached transcription for the given audio hash.
pub fn cached_transcription(audio_hash: &str) -> Option<String> {
    let path = cache_path(audio_hash).ok()?;
    std::fs::read_to_string(path).ok()
}

/// Save transcription text to cache.
pub fn save_transcription_cache(audio_hash: &str, text: &str) -> Result<()> {
    let path = cache_path(audio_hash)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("failed to create cache directory")?;
    }
    std::fs::write(&path, text).context("failed to write transcription cache")?;
    Ok(())
}

/// Pick a reasonable thread count for Whisper.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
fn num_threads() -> i32 {
    let cpus = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(4);
    // Use at most 4 threads to avoid starving the UI
    cpus.min(4) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn models_dir_is_under_data() {
        let dir = models_dir().expect("models_dir");
        assert!(dir.ends_with("qvox/models"));
    }

    #[test]
    fn default_model_path_has_filename() {
        let path = default_model_path().expect("model path");
        assert_eq!(path.file_name().and_then(|n| n.to_str()), Some("ggml-base.bin"));
    }

    #[test]
    fn cache_path_uses_hash() {
        let path = cache_path("abc123").expect("cache path");
        assert!(path.to_string_lossy().contains("abc123.txt"));
    }

    #[test]
    fn cached_transcription_returns_none_for_missing() {
        assert!(cached_transcription("nonexistent_hash_12345").is_none());
    }

    #[test]
    fn save_and_read_cache() {
        let hash = "test_cache_round_trip_qvox";
        let text = "Hello world transcription";

        save_transcription_cache(hash, text).expect("save");
        let cached = cached_transcription(hash);
        assert_eq!(cached.as_deref(), Some(text));

        // Cleanup
        if let Ok(path) = cache_path(hash) {
            std::fs::remove_file(path).ok();
        }
    }

    #[test]
    fn linear_resample_identity() {
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let output = linear_resample(&input, 16_000, 16_000);
        assert_eq!(input, output);
    }

    #[test]
    fn linear_resample_empty() {
        let output = linear_resample(&[], 44_100, 16_000);
        assert!(output.is_empty());
    }

    #[test]
    fn linear_resample_downsamples() {
        let input = vec![0.0, 1.0, 0.0, -1.0];
        let output = linear_resample(&input, 44_100, 16_000);
        assert!(output.len() < input.len(), "expected downsampled output");
    }

    #[test]
    fn linear_resample_upsamples() {
        let input = vec![0.0, 1.0];
        let output = linear_resample(&input, 8_000, 16_000);
        assert!(output.len() > input.len(), "expected upsampled output");
    }

    #[test]
    fn load_wav_16khz_mono_valid() {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut buf = Cursor::new(Vec::new());
        {
            let mut writer = hound::WavWriter::new(&mut buf, spec).expect("create writer");
            for i in 0_i16..1600 {
                #[allow(clippy::cast_possible_truncation)]
                let sample = ((f32::from(i) / 1600.0) * 32767.0) as i16;
                writer.write_sample(sample).expect("write sample");
            }
            writer.finalize().expect("finalize");
        }

        let wav_bytes = buf.into_inner();
        let samples = load_wav_16khz_mono(&wav_bytes).expect("load wav");
        assert_eq!(samples.len(), 1600);
    }

    #[test]
    fn load_wav_16khz_mono_stereo_to_mono() {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 16_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut buf = Cursor::new(Vec::new());
        {
            let mut writer = hound::WavWriter::new(&mut buf, spec).expect("create writer");
            for _ in 0..100 {
                writer.write_sample(1000_i16).expect("left");
                writer.write_sample(-1000_i16).expect("right");
            }
            writer.finalize().expect("finalize");
        }

        let wav_bytes = buf.into_inner();
        let samples = load_wav_16khz_mono(&wav_bytes).expect("load wav");
        assert_eq!(samples.len(), 100);
        for &s in &samples {
            assert!(s.abs() < 0.01, "expected near-zero mono mix, got {s}");
        }
    }

    #[test]
    fn num_threads_reasonable() {
        let n = num_threads();
        assert!((1..=4).contains(&n));
    }
}

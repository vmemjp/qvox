/// Normalize audio samples to a target peak level in dB.
///
/// Computes the peak amplitude of `samples`, then scales all values so the
/// peak matches `target_db`.  If the signal is silent (peak == 0) the
/// buffer is left unchanged.
pub fn normalize_audio(samples: &mut [f32], target_db: f32) {
    let peak = samples
        .iter()
        .copied()
        .map(f32::abs)
        .fold(0.0_f32, f32::max);

    if peak == 0.0 {
        return;
    }

    let target_linear = 10.0_f32.powf(target_db / 20.0);
    let gain = target_linear / peak;

    for s in samples.iter_mut() {
        *s *= gain;
    }
}

/// Truncate samples to at most `max_seconds` of audio.
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn trim_to_max_duration(samples: &mut Vec<f32>, sample_rate: u32, max_seconds: f32) {
    let max_frames = (max_seconds * sample_rate as f32) as usize;
    if samples.len() > max_frames {
        samples.truncate(max_frames);
    }
}

/// Remove trailing silence from audio samples.
///
/// Walks backward in 100 ms chunks, computing the RMS of each.  Chunks
/// whose RMS falls below `threshold_db` are trimmed off.
///
/// Constraints:
/// - At least 5 seconds of audio are always preserved.
/// - A 200 ms tail buffer is kept after trimming so the audio doesn't end
///   too abruptly.
pub fn remove_trailing_silence(
    samples: &mut Vec<f32>,
    sample_rate: u32,
    threshold_db: f32,
) {
    let chunk_size = (sample_rate as usize) / 10; // 100 ms
    let min_frames = (sample_rate as usize) * 5; // 5 seconds
    let tail_buffer = (sample_rate as usize) / 5; // 200 ms

    if samples.len() <= min_frames {
        return;
    }

    let threshold_linear = 10.0_f32.powf(threshold_db / 20.0);

    let mut end = samples.len();

    while end > min_frames {
        let start = end.saturating_sub(chunk_size).max(min_frames);
        let chunk = &samples[start..end];
        let rms = rms_level(chunk);

        if rms > threshold_linear {
            break;
        }
        end = start;
    }

    // Add 200 ms tail buffer back (don't exceed original length)
    let final_len = (end + tail_buffer).min(samples.len());
    samples.truncate(final_len);
}

/// Compute the RMS level of a slice of f32 samples.
fn rms_level(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let mean_sq = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
    mean_sq.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_silent_signal() {
        let mut samples = vec![0.0; 100];
        normalize_audio(&mut samples, -3.0);
        assert!(samples.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn normalize_scales_to_target() {
        let mut samples = vec![0.5, -0.5, 0.25, -0.25];
        normalize_audio(&mut samples, -6.0);
        let peak = samples.iter().copied().map(f32::abs).fold(0.0_f32, f32::max);
        let expected = 10.0_f32.powf(-6.0 / 20.0);
        assert!((peak - expected).abs() < 1e-5, "peak={peak}, expected={expected}");
    }

    #[test]
    fn normalize_to_zero_db() {
        let mut samples = vec![0.3, -0.7, 0.1];
        normalize_audio(&mut samples, 0.0);
        let peak = samples.iter().copied().map(f32::abs).fold(0.0_f32, f32::max);
        assert!((peak - 1.0).abs() < 1e-5);
    }

    #[test]
    fn trim_shortens_long_audio() {
        let mut samples = vec![1.0; 48_000];
        trim_to_max_duration(&mut samples, 48_000, 0.5);
        assert_eq!(samples.len(), 24_000);
    }

    #[test]
    fn trim_preserves_short_audio() {
        let mut samples = vec![1.0; 1_000];
        trim_to_max_duration(&mut samples, 48_000, 60.0);
        assert_eq!(samples.len(), 1_000);
    }

    #[test]
    fn remove_silence_trims_trailing_zeros() {
        let sample_rate = 16_000_u32;
        let mut samples = Vec::new();
        samples.extend(vec![0.5; (sample_rate as usize) * 10]);
        samples.extend(vec![0.0; (sample_rate as usize) * 3]);

        remove_trailing_silence(&mut samples, sample_rate, -40.0);

        let expected_approx = (sample_rate as usize) * 10 + (sample_rate as usize) / 5;
        assert!(
            samples.len() <= expected_approx + 1600,
            "len={}, expected_approx={expected_approx}",
            samples.len()
        );
        assert!(
            samples.len() >= (sample_rate as usize) * 10,
            "should preserve at least 10s of signal, got {}",
            samples.len()
        );
    }

    #[test]
    fn remove_silence_preserves_minimum_duration() {
        let sample_rate = 16_000_u32;
        let mut samples = vec![0.0; (sample_rate as usize) * 4];
        let original_len = samples.len();

        remove_trailing_silence(&mut samples, sample_rate, -40.0);

        assert_eq!(samples.len(), original_len);
    }

    #[test]
    fn remove_silence_no_silence_at_end() {
        let sample_rate = 16_000_u32;
        let mut samples = vec![0.5; (sample_rate as usize) * 8];
        let original_len = samples.len();

        remove_trailing_silence(&mut samples, sample_rate, -40.0);

        assert_eq!(samples.len(), original_len);
    }

    #[test]
    fn rms_of_empty() {
        assert!(rms_level(&[]).abs() < f32::EPSILON);
    }

    #[test]
    fn rms_of_constant() {
        let samples = vec![0.5; 100];
        let rms = rms_level(&samples);
        assert!((rms - 0.5).abs() < 1e-5);
    }
}

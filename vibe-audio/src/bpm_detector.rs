use crate::{fetcher::Fetcher, SampleProcessor};

/// Configuration for the BPM detector.
#[derive(Debug, Clone)]
pub struct BpmDetectorConfig {
    /// Size of the onset history buffer in seconds (default: 15.0)
    pub history_seconds: f32,
    /// Minimum BPM to detect (default: 60)
    pub min_bpm: f32,
    /// Maximum BPM to detect (default: 200)
    pub max_bpm: f32,
    /// Number of BPM estimates to keep for median calculation (default: 60)
    /// At 15-second intervals, 60 estimates = 15 minutes of history
    pub estimate_history_size: usize,
}

impl Default for BpmDetectorConfig {
    fn default() -> Self {
        Self {
            history_seconds: 15.0,  // 15 seconds of audio history per estimate
            min_bpm: 60.0,
            max_bpm: 200.0,
            estimate_history_size: 60,  // 60 estimates * 15 seconds = 15 minutes
        }
    }
}

/// Detects BPM from audio using spectral flux and autocorrelation.
///
/// The detector analyzes bass frequencies (20-200 Hz) to find onsets,
/// then uses autocorrelation to find the periodic tempo pattern.
/// BPM is computed as the median of multiple estimates over time,
/// providing stability against transient percussion.
pub struct BpmDetector {
    config: BpmDetectorConfig,

    // Onset detection state
    prev_bass_energy: f32,
    onset_history: Box<[f32]>,
    onset_write_idx: usize,

    // BPM estimate history for median calculation
    bpm_estimates: Vec<f32>,
    current_bpm: f32,
    frames_per_second: f32,

    // Update throttling - only compute BPM every N frames
    frame_count: usize,
    frames_between_updates: usize,

    // Bass frequency bin range in FFT output
    bass_bin_start: usize,
    bass_bin_end: usize,
}

impl BpmDetector {
    /// Creates a new BPM detector.
    ///
    /// The detector needs the SampleProcessor to determine sample rate and FFT size
    /// for calculating frequency bin ranges and timing.
    pub fn new<F: Fetcher>(processor: &SampleProcessor<F>, config: BpmDetectorConfig) -> Self {
        let sample_rate = processor.sample_rate() as f32;
        let fft_size = processor.fft_size();
        let frames_per_second = sample_rate / fft_size as f32;

        // Calculate onset buffer size (number of frames to store)
        let history_frames = (config.history_seconds * frames_per_second) as usize;
        let onset_history = vec![0.0f32; history_frames].into_boxed_slice();

        // Calculate bass frequency bin range (20-200 Hz)
        let freq_resolution = sample_rate / fft_size as f32;
        let bass_bin_start = (20.0f32 / freq_resolution).ceil() as usize;
        let bass_bin_end = (200.0f32 / freq_resolution).ceil() as usize;

        // Only update BPM every ~15 seconds
        let frames_between_updates = (frames_per_second * 15.0) as usize;

        Self {
            config,
            prev_bass_energy: 0.0,
            onset_history,
            onset_write_idx: 0,
            bpm_estimates: Vec::with_capacity(60),
            current_bpm: 120.0, // Default starting BPM
            frames_per_second,
            frame_count: 0,
            frames_between_updates,
            bass_bin_start,
            bass_bin_end,
        }
    }

    /// Process a new audio frame and return the current BPM estimate.
    ///
    /// This should be called once per frame after `SampleProcessor::process_next_samples()`.
    pub fn process<F: Fetcher>(&mut self, processor: &SampleProcessor<F>) -> f32 {
        let fft_out = processor.fft_out();
        if fft_out.is_empty() {
            return self.current_bpm;
        }

        // Use first channel for BPM detection
        let fft_data = &fft_out[0].fft_out;

        // Ensure bin range is valid
        let bin_end = self.bass_bin_end.min(fft_data.len());
        let bin_start = self.bass_bin_start.min(bin_end);

        if bin_start >= bin_end {
            return self.current_bpm;
        }

        // 1. Calculate bass energy from FFT output
        let bass_energy: f32 = fft_data[bin_start..bin_end]
            .iter()
            .map(|c| c.norm())
            .sum();

        // 2. Spectral flux (positive energy changes indicate onsets)
        let flux = (bass_energy - self.prev_bass_energy).max(0.0);
        self.prev_bass_energy = bass_energy;

        // 3. Store in onset history (circular buffer)
        self.onset_history[self.onset_write_idx] = flux;
        self.onset_write_idx = (self.onset_write_idx + 1) % self.onset_history.len();

        // 4. Only compute BPM periodically (not every frame)
        self.frame_count += 1;
        if self.frame_count >= self.frames_between_updates {
            self.frame_count = 0;

            let detected_bpm = self.compute_bpm_from_autocorrelation();

            // Add to estimate history
            if detected_bpm >= self.config.min_bpm && detected_bpm <= self.config.max_bpm {
                self.bpm_estimates.push(detected_bpm);

                // Keep only the most recent estimates
                if self.bpm_estimates.len() > self.config.estimate_history_size {
                    self.bpm_estimates.remove(0);
                }
            }

            // Update current BPM to median of estimates
            if !self.bpm_estimates.is_empty() {
                self.current_bpm = self.compute_median_bpm();
            }
        }

        self.current_bpm
    }

    /// Returns the current smoothed BPM estimate.
    pub fn bpm(&self) -> f32 {
        self.current_bpm
    }

    /// Compute the median of all BPM estimates.
    fn compute_median_bpm(&self) -> f32 {
        if self.bpm_estimates.is_empty() {
            return self.current_bpm;
        }

        let mut sorted: Vec<f32> = self.bpm_estimates.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            // Even number of elements: average the two middle values
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            // Odd number of elements: take the middle value
            sorted[mid]
        }
    }

    /// Compute BPM by finding the strongest periodic pattern in onset history.
    fn compute_bpm_from_autocorrelation(&self) -> f32 {
        // Convert BPM range to lag range in frames
        // lag = frames_per_second * (60 / bpm)
        let min_lag = (60.0 / self.config.max_bpm * self.frames_per_second) as usize;
        let max_lag = (60.0 / self.config.min_bpm * self.frames_per_second) as usize;

        // Ensure we don't exceed half the buffer (autocorrelation requirement)
        let max_lag = max_lag.min(self.onset_history.len() / 2);

        if min_lag >= max_lag {
            return self.current_bpm;
        }

        let mut best_lag = min_lag;
        let mut best_correlation = 0.0f32;

        // Find the lag with the highest autocorrelation
        for lag in min_lag..max_lag {
            let correlation = self.autocorrelation(lag);
            if correlation > best_correlation {
                best_correlation = correlation;
                best_lag = lag;
            }
        }

        // Convert lag back to BPM
        if best_lag > 0 {
            60.0 * self.frames_per_second / best_lag as f32
        } else {
            self.current_bpm
        }
    }

    /// Compute autocorrelation at a given lag.
    fn autocorrelation(&self, lag: usize) -> f32 {
        let len = self.onset_history.len();
        if lag >= len {
            return 0.0;
        }

        let mut sum = 0.0;
        let count = len - lag;

        // Since we're using a circular buffer, we need to handle wrap-around
        // For simplicity, we treat the buffer as linear starting from write position
        for i in 0..count {
            let idx1 = (self.onset_write_idx + len - count + i) % len;
            let idx2 = (idx1 + lag) % len;
            sum += self.onset_history[idx1] * self.onset_history[idx2];
        }

        sum / count as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = BpmDetectorConfig::default();
        assert_eq!(config.history_seconds, 15.0);
        assert_eq!(config.min_bpm, 60.0);
        assert_eq!(config.max_bpm, 200.0);
        assert_eq!(config.estimate_history_size, 60);
    }
}

use rustfft::{num_complex::Complex, FftPlanner};

pub struct SignalProcessor {
    fft_size: usize,
    planner: FftPlanner<f32>,
    window: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoundType {
    Silence,
    Music,
    Voice,
    Noise,
}

pub struct SignalData {
    pub rms: f32,
    pub fft: Vec<f32>,
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub sound_type: SoundType,
}

impl SignalProcessor {
    pub fn new(fft_size: usize) -> Self {
        Self {
            fft_size,
            planner: FftPlanner::new(),
            window: hann_window(fft_size),
        }
    }

    pub fn process(&mut self, samples: &[f32]) -> SignalData {
        if samples.is_empty() {
            return SignalData {
                rms: 0.0,
                fft: vec![0.0; self.fft_size / 2],
                bass: 0.0,
                mid: 0.0,
                treble: 0.0,
                sound_type: SoundType::Silence,
            };
        }

        // 1. RMS
        let mut sum_sq = 0.0;
        for &s in samples {
            sum_sq += s * s;
        }
        let rms = (sum_sq / samples.len() as f32).sqrt();

        // 2. FFT with a Hann window to reduce spectral leakage.
        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .take(self.fft_size)
            .enumerate()
            .map(|(i, &s)| Complex {
                re: s * self.window[i],
                im: 0.0,
            })
            .collect();

        if buffer.len() < self.fft_size {
            buffer.resize(self.fft_size, Complex { re: 0.0, im: 0.0 });
        }

        let fft = self.planner.plan_fft_forward(self.fft_size);
        fft.process(&mut buffer);

        let magnitudes: Vec<f32> = buffer
            .iter()
            .take(self.fft_size / 2)
            .map(|c| normalize_magnitude((c.re * c.re + c.im * c.im).sqrt(), self.fft_size))
            .collect();

        let fft = logarithmic_bands(&magnitudes, self.fft_size / 2);
        let bass = band_average(&magnitudes, 1, 32);
        let mid = band_average(&magnitudes, 32, 160);
        let treble = band_average(&magnitudes, 160, magnitudes.len());

        // 3. Simple classification heuristics over normalized bands.
        let sound_type = if rms < 0.005 {
            SoundType::Silence
        } else if treble > mid * 1.2 && treble > bass * 1.2 {
            SoundType::Noise
        } else if mid > bass * 0.9 && mid > treble * 1.2 {
            SoundType::Voice
        } else {
            SoundType::Music
        };

        SignalData {
            rms,
            fft,
            bass,
            mid,
            treble,
            sound_type,
        }
    }
}

fn hann_window(size: usize) -> Vec<f32> {
    if size <= 1 {
        return vec![1.0; size];
    }

    (0..size)
        .map(|i| {
            0.5 - 0.5
                * ((2.0 * std::f32::consts::PI * i as f32) / (size.saturating_sub(1)) as f32).cos()
        })
        .collect()
}

fn normalize_magnitude(magnitude: f32, fft_size: usize) -> f32 {
    let linear = magnitude / (fft_size as f32 * 0.5);
    (linear * 16.0).ln_1p() / 17.0_f32.ln()
}

fn logarithmic_bands(magnitudes: &[f32], output_len: usize) -> Vec<f32> {
    if magnitudes.is_empty() || output_len == 0 {
        return Vec::new();
    }

    let max_bin = magnitudes.len();
    let log_max = (max_bin as f32 + 1.0).ln();
    (0..output_len)
        .map(|i| {
            let start = ((i as f32 / output_len as f32) * log_max).exp() as usize;
            let end =
                ((((i + 1) as f32 / output_len as f32) * log_max).exp() as usize).max(start + 1);
            band_average(magnitudes, start.saturating_sub(1), end.min(max_bin))
        })
        .collect()
}

fn band_average(magnitudes: &[f32], start: usize, end: usize) -> f32 {
    if magnitudes.is_empty() || start >= magnitudes.len() {
        return 0.0;
    }

    let end = end.min(magnitudes.len()).max(start + 1);
    magnitudes[start..end].iter().sum::<f32>() / (end - start) as f32
}

#[cfg(test)]
mod tests {
    use super::{band_average, hann_window, logarithmic_bands, SignalProcessor, SoundType};

    #[test]
    fn hann_window_tapers_edges() {
        let window = hann_window(8);

        assert!(window[0] < 0.001);
        assert!(window[3] > 0.9);
        assert!(window[7] < 0.001);
    }

    #[test]
    fn process_returns_normalized_log_bands() {
        let mut processor = SignalProcessor::new(1024);
        let samples: Vec<f32> = (0..1024)
            .map(|i| (2.0 * std::f32::consts::PI * i as f32 / 32.0).sin())
            .collect();

        let signal = processor.process(&samples);

        assert_eq!(signal.fft.len(), 512);
        assert!(signal.rms > 0.6);
        assert!(signal.fft.iter().all(|v| *v >= 0.0 && *v <= 1.0));
        assert_ne!(signal.sound_type, SoundType::Silence);
    }

    #[test]
    fn empty_input_returns_silence() {
        let mut processor = SignalProcessor::new(1024);
        let signal = processor.process(&[]);

        assert_eq!(signal.rms, 0.0);
        assert_eq!(signal.bass, 0.0);
        assert_eq!(signal.mid, 0.0);
        assert_eq!(signal.treble, 0.0);
        assert_eq!(signal.sound_type, SoundType::Silence);
    }

    #[test]
    fn band_average_handles_out_of_range_requests() {
        let magnitudes = [0.2, 0.4, 0.6];

        assert_eq!(band_average(&magnitudes, 1, 3), 0.5);
        assert_eq!(band_average(&magnitudes, 10, 12), 0.0);
    }

    #[test]
    fn logarithmic_bands_preserve_requested_length() {
        let magnitudes = vec![0.1; 128];

        assert_eq!(logarithmic_bands(&magnitudes, 64).len(), 64);
    }
}

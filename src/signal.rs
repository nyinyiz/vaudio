use rustfft::{num_complex::Complex, FftPlanner};

pub struct SignalProcessor {
    fft_size: usize,
    planner: FftPlanner<f32>,
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
    pub sound_type: SoundType,
}

impl SignalProcessor {
    pub fn new(fft_size: usize) -> Self {
        Self {
            fft_size,
            planner: FftPlanner::new(),
        }
    }

    pub fn process(&mut self, samples: &[f32]) -> SignalData {
        if samples.is_empty() {
            return SignalData {
                rms: 0.0,
                fft: vec![0.0; self.fft_size / 2],
                sound_type: SoundType::Silence,
            };
        }

        // 1. RMS
        let mut sum_sq = 0.0;
        for &s in samples {
            sum_sq += s * s;
        }
        let rms = (sum_sq / samples.len() as f32).sqrt();

        // 2. FFT
        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .take(self.fft_size)
            .map(|&s| Complex { re: s, im: 0.0 })
            .collect();

        if buffer.len() < self.fft_size {
            buffer.resize(self.fft_size, Complex { re: 0.0, im: 0.0 });
        }

        let fft = self.planner.plan_fft_forward(self.fft_size);
        fft.process(&mut buffer);

        let magnitudes: Vec<f32> = buffer
            .iter()
            .take(self.fft_size / 2)
            .map(|c| (c.re * c.re + c.im * c.im).sqrt())
            .collect();

        // 3. Simple Classification Heuristics
        let sound_type = if rms < 0.005 {
            SoundType::Silence
        } else {
            let low_end: f32 = magnitudes.iter().take(10).sum::<f32>() / 10.0;
            let mid_range: f32 = magnitudes.iter().skip(10).take(50).sum::<f32>() / 50.0;
            let high_end: f32 = magnitudes.iter().skip(100).sum::<f32>() / (magnitudes.len() - 100) as f32;

            if low_end > mid_range * 2.0 && low_end > 0.1 {
                SoundType::Music // Strong bass usually means music/beat
            } else if mid_range > high_end * 1.5 {
                SoundType::Voice // Speech is mid-heavy
            } else if high_end > mid_range * 0.5 {
                SoundType::Noise // Constant hiss or wide-band noise
            } else {
                SoundType::Music // Default to music if complex
            }
        };

        SignalData {
            rms,
            fft: magnitudes,
            sound_type,
        }
    }
}

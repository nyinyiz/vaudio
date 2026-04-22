use rustfft::{num_complex::Complex, FftPlanner};

pub struct SignalProcessor {
    fft_size: usize,
    planner: FftPlanner<f32>,
}

pub struct SignalData {
    pub rms: f32,
    pub peak: f32,
    pub fft: Vec<f32>,
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
                peak: 0.0,
                fft: vec![0.0; self.fft_size / 2],
            };
        }

        // 1. RMS and Peak
        let mut sum_sq = 0.0;
        let mut max_abs: f32 = 0.0;
        for &s in samples {
            sum_sq += s * s;
            max_abs = max_abs.max(s.abs());
        }
        let rms = (sum_sq / samples.len() as f32).sqrt();

        // 2. FFT
        // Use a window if needed, but for visualization direct is often okay if buffer is large enough
        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .take(self.fft_size)
            .map(|&s| Complex { re: s, im: 0.0 })
            .collect();

        // Pad with zeros if necessary
        if buffer.len() < self.fft_size {
            buffer.resize(self.fft_size, Complex { re: 0.0, im: 0.0 });
        }

        let fft = self.planner.plan_fft_forward(self.fft_size);
        fft.process(&mut buffer);

        // Compute magnitudes for the first half (Nyquist)
        let magnitudes: Vec<f32> = buffer
            .iter()
            .take(self.fft_size / 2)
            .map(|c| (c.re * c.re + c.im * c.im).sqrt())
            .collect();

        SignalData {
            rms,
            peak: max_abs,
            fft: magnitudes,
        }
    }
}

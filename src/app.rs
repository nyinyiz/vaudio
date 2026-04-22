use crate::render::ViewMode;
use crate::render::rain::RainDrop;
use crate::signal::SignalProcessor;
use rand::Rng;

pub struct App {
    pub mode: ViewMode,
    pub sensitivity: f32,
    pub mirror: bool,
    pub no_color: bool,
    
    // Audio State
    pub processor: SignalProcessor,
    pub fft_data: Vec<f32>,
    pub peaks: Vec<f32>,
    pub wave_data: Vec<f32>,
    pub rms: f32,
    
    // Rain State
    pub rain_drops: Vec<RainDrop>,
    
    // Smoothing factors
    pub smoothing: f32,
}

impl App {
    pub fn new(mode: ViewMode, sensitivity: f32, mirror: bool, no_color: bool) -> Self {
        let fft_size = 1024;
        Self {
            mode,
            sensitivity,
            mirror,
            no_color,
            processor: SignalProcessor::new(fft_size),
            fft_data: vec![0.0; fft_size / 2],
            peaks: vec![0.0; fft_size / 2],
            wave_data: vec![0.0; 512],
            rms: 0.0,
            rain_drops: Vec::new(),
            smoothing: 0.7,
        }
    }

    pub fn update_audio(&mut self, samples: &[f32], width: u16, height: u16) {
        let signal = self.processor.process(samples);
        
        // Update RMS with sensitivity
        self.rms = (signal.rms * self.sensitivity).min(1.0);
        
        // Update Wave (keep a rolling buffer)
        self.wave_data.extend_from_slice(samples);
        if self.wave_data.len() > 1024 {
            let start = self.wave_data.len() - 1024;
            self.wave_data.drain(0..start);
        }

        // Update FFT with smoothing and sensitivity
        for (i, &new_val) in signal.fft.iter().enumerate() {
            if i >= self.fft_data.len() { break; }
            let val = (new_val * self.sensitivity).min(1.0);
            self.fft_data[i] = self.fft_data[i] * self.smoothing + val * (1.0 - self.smoothing);
            
            // Peak decay
            if self.fft_data[i] > self.peaks[i] {
                self.peaks[i] = self.fft_data[i];
            } else {
                self.peaks[i] *= 0.95;
            }
        }

        // Update Rain drops logic
        self.update_rain(width, height);
    }

    fn update_rain(&mut self, width: u16, height: u16) {
        let mut rng = rand::thread_rng();
        
        // Higher spawn rate and use actual terminal width
        let spawn_chance = (self.rms * 3.0).clamp(0.05, 0.9);
        if width > 0 && rng.gen_bool(spawn_chance as f64) {
            let x = rng.gen_range(0..width);
            self.rain_drops.push(RainDrop {
                x,
                y: 0.0,
                speed: rng.gen_range(0.2..0.6) + self.rms * 1.0,
                length: rng.gen_range(5..15),
                chars: (0..20).map(|_| rng.gen_range(33..126) as u8 as char).collect(),
            });
        }

        // Move and filter drops
        for drop in &mut self.rain_drops {
            drop.y += drop.speed;
        }
        self.rain_drops.retain(|d| d.y < height as f32 + 20.0);
    }

    pub fn set_mode(&mut self, mode: ViewMode) {
        self.mode = mode;
    }

    pub fn adjust_sensitivity(&mut self, delta: f32) {
        self.sensitivity = (self.sensitivity + delta).max(0.1).min(10.0);
    }
}

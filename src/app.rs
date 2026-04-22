use crate::render::ViewMode;
use crate::render::rain::RainDrop;
use crate::signal::SignalProcessor;
use rand::Rng;

pub struct PulseRing {
    pub radius: f32,
    pub intensity: f32,
}

pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
}

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
    
    // Mode State
    pub rain_drops: Vec<RainDrop>,
    pub pulse_rings: Vec<PulseRing>,
    pub spectrogram_history: Vec<Vec<f32>>,
    pub spinner_angle: f32,
    pub particles: Vec<Particle>,
    
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
            pulse_rings: Vec::new(),
            spectrogram_history: Vec::new(),
            spinner_angle: 0.0,
            particles: Vec::new(),
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

        // Update Mode logic
        self.update_rain(width, height);
        self.update_pulse();
        self.update_spectrogram(height);
        self.update_spinner();
        self.update_particles(width, height);
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

    fn update_pulse(&mut self) {
        // Spawn a new ring if RMS is high enough
        if self.rms > 0.05 {
            self.pulse_rings.push(PulseRing {
                radius: 0.0,
                intensity: self.rms,
            });
        }

        // Expand and filter rings
        for ring in &mut self.pulse_rings {
            ring.radius += 0.5 + ring.intensity * 1.5;
        }
        self.pulse_rings.retain(|r| r.radius < 200.0);
    }

    fn update_spectrogram(&mut self, height: u16) {
        // Save current FFT to history
        self.spectrogram_history.insert(0, self.fft_data.clone());
        
        // Limit history to terminal height
        if self.spectrogram_history.len() > height as usize {
            self.spectrogram_history.truncate(height as usize);
        }
    }

    fn update_spinner(&mut self) {
        // Speed up based on RMS
        self.spinner_angle += 0.05 + self.rms * 0.2;
        if self.spinner_angle > std::f32::consts::TAU {
            self.spinner_angle -= std::f32::consts::TAU;
        }
    }

    fn update_particles(&mut self, width: u16, height: u16) {
        let mut rng = rand::thread_rng();
        
        // Spawn particles on high RMS
        if self.rms > 0.05 {
            let num_new = (self.rms * 30.0) as usize;
            for _ in 0..num_new {
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let speed = rng.gen_range(0.2..1.5) + self.rms * 2.0;
                self.particles.push(Particle {
                    x: width as f32 / 2.0,
                    y: height as f32 / 2.0,
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed * 0.5, // Aspect ratio correction
                    life: 1.0,
                });
            }
        }

        // Move and age particles
        for p in &mut self.particles {
            p.x += p.vx;
            p.y += p.vy;
            p.life -= 0.01; // Slower aging
        }
        self.particles.retain(|p| p.life > 0.0 && p.x >= -10.0 && p.x < width as f32 + 10.0 && p.y >= -10.0 && p.y < height as f32 + 10.0);
    }

    pub fn set_mode(&mut self, mode: ViewMode) {
        self.mode = mode;
    }

    pub fn adjust_sensitivity(&mut self, delta: f32) {
        self.sensitivity = (self.sensitivity + delta).max(0.1).min(10.0);
    }
}

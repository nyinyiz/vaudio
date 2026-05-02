use crate::render::rain::RainDrop;
use crate::render::ViewMode;
use crate::signal::{SignalProcessor, SoundType};
use crate::theme::Theme;
use rand::Rng;
use std::collections::VecDeque;

const BEAT_HISTORY_LEN: usize = 43;
const BEAT_MIN_HISTORY: usize = 16;
const BEAT_COOLDOWN_FRAMES: u8 = 6;
const AUTO_SWITCH_COOLDOWN_FRAMES: u8 = 18;

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
    pub theme: Theme,
    pub auto_mode: ViewMode,
    pub last_manual_mode: ViewMode,

    // Audio State
    pub processor: SignalProcessor,
    pub fft_data: Vec<f32>,
    pub peaks: Vec<f32>,
    pub wave_data: Vec<f32>,
    pub rms: f32,
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub beat: bool,
    pub beat_intensity: f32,
    pub sound_type: SoundType,

    // Mode State
    pub rain_drops: Vec<RainDrop>,
    pub pulse_rings: Vec<PulseRing>,
    pub spectrogram_history: Vec<Vec<f32>>,
    pub spinner_angle: f32,
    pub particles: Vec<Particle>,

    // Smoothing factors
    pub smoothing: f32,
    energy_history: VecDeque<f32>,
    beat_cooldown: u8,
    auto_switch_cooldown: u8,
}

impl App {
    pub fn new(
        mode: ViewMode,
        sensitivity: f32,
        mirror: bool,
        no_color: bool,
        theme: Theme,
    ) -> Self {
        let fft_size = 1024;
        Self {
            mode,
            sensitivity,
            mirror,
            no_color,
            theme,
            auto_mode: ViewMode::Spinner,
            last_manual_mode: if mode == ViewMode::Auto {
                ViewMode::Bars
            } else {
                mode
            },
            processor: SignalProcessor::new(fft_size),
            fft_data: vec![0.0; fft_size / 2],
            peaks: vec![0.0; fft_size / 2],
            wave_data: vec![0.0; 512],
            rms: 0.0,
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            beat: false,
            beat_intensity: 0.0,
            sound_type: SoundType::Silence,
            rain_drops: Vec::new(),
            pulse_rings: Vec::new(),
            spectrogram_history: Vec::new(),
            spinner_angle: 0.0,
            particles: Vec::new(),
            smoothing: 0.7,
            energy_history: VecDeque::with_capacity(BEAT_HISTORY_LEN),
            beat_cooldown: 0,
            auto_switch_cooldown: 0,
        }
    }

    pub fn update_audio(&mut self, samples: &[f32], width: u16, height: u16) {
        let signal = self.processor.process(samples);
        self.sound_type = signal.sound_type;

        // Update RMS with sensitivity
        self.rms = (signal.rms * self.sensitivity).min(1.0);
        self.bass = (signal.bass * self.sensitivity).min(1.0);
        self.mid = (signal.mid * self.sensitivity).min(1.0);
        self.treble = (signal.treble * self.sensitivity).min(1.0);
        self.detect_beat(self.rms * 0.35 + self.bass * 0.65);
        self.update_auto_mode();

        // Update Wave (keep a rolling buffer)
        self.wave_data.extend_from_slice(samples);
        if self.wave_data.len() > 1024 {
            let start = self.wave_data.len() - 1024;
            self.wave_data.drain(0..start);
        }

        // Update FFT with smoothing and sensitivity
        for (i, &new_val) in signal.fft.iter().enumerate() {
            if i >= self.fft_data.len() {
                break;
            }
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
        let beat_boost = if self.beat {
            self.beat_intensity * 0.35
        } else {
            0.0
        };
        let spawn_chance = (self.rms * 2.0 + beat_boost).clamp(0.03, 0.9);
        if width > 0 && rng.gen_bool(spawn_chance as f64) {
            let x = rng.gen_range(0..width);
            self.rain_drops.push(RainDrop {
                x,
                y: 0.0,
                speed: rng.gen_range(0.2..0.6) + self.rms * 1.0,
                length: rng.gen_range(5..15),
                chars: (0..20)
                    .map(|_| rng.gen_range(33..126) as u8 as char)
                    .collect(),
            });
        }

        // Move and filter drops
        for drop in &mut self.rain_drops {
            drop.y += drop.speed;
        }
        self.rain_drops.retain(|d| d.y < height as f32 + 20.0);
    }

    fn update_pulse(&mut self) {
        // Spawn a new ring on transients instead of every loud frame.
        if self.beat {
            self.pulse_rings.push(PulseRing {
                radius: 0.0,
                intensity: self.beat_intensity.max(self.rms),
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
        let beat_kick = if self.beat {
            self.beat_intensity * 0.25
        } else {
            0.0
        };
        self.spinner_angle += 0.05 + self.rms * 0.15 + beat_kick;
        if self.spinner_angle > std::f32::consts::TAU {
            self.spinner_angle -= std::f32::consts::TAU;
        }
    }

    fn update_particles(&mut self, width: u16, height: u16) {
        let mut rng = rand::thread_rng();

        // Spawn particles as distinct bursts on beat transients.
        if self.beat {
            let num_new = (10.0 + self.beat_intensity * 50.0) as usize;
            for _ in 0..num_new {
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let speed = rng.gen_range(0.2..1.5) + self.beat_intensity * 2.0;
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
        self.particles.retain(|p| {
            p.life > 0.0
                && p.x >= -10.0
                && p.x < width as f32 + 10.0
                && p.y >= -10.0
                && p.y < height as f32 + 10.0
        });
    }

    pub fn set_mode(&mut self, mode: ViewMode) {
        if mode != ViewMode::Auto {
            self.last_manual_mode = mode;
        }
        self.mode = mode;
    }

    pub fn toggle_auto_manual(&mut self) {
        if self.mode == ViewMode::Auto {
            self.mode = self.last_manual_mode;
        } else {
            self.last_manual_mode = self.mode;
            self.mode = ViewMode::Auto;
        }
    }

    pub fn active_mode(&self) -> ViewMode {
        if self.mode == ViewMode::Auto {
            self.auto_mode
        } else {
            self.mode
        }
    }

    pub fn adjust_sensitivity(&mut self, delta: f32) {
        self.sensitivity = (self.sensitivity + delta).max(0.1).min(10.0);
    }

    pub fn cycle_theme(&mut self) {
        self.theme = self.theme.next();
    }

    fn detect_beat(&mut self, energy: f32) {
        let avg_energy = if self.energy_history.is_empty() {
            0.0
        } else {
            self.energy_history.iter().sum::<f32>() / self.energy_history.len() as f32
        };
        let threshold = avg_energy * 1.45 + 0.03;

        self.beat = self.energy_history.len() >= BEAT_MIN_HISTORY
            && self.beat_cooldown == 0
            && energy > threshold
            && energy > 0.05;
        self.beat_intensity = if self.beat {
            (energy / threshold).clamp(0.0, 2.0) / 2.0
        } else {
            0.0
        };

        if self.beat {
            self.beat_cooldown = BEAT_COOLDOWN_FRAMES;
        } else {
            self.beat_cooldown = self.beat_cooldown.saturating_sub(1);
        }

        self.energy_history.push_back(energy);
        if self.energy_history.len() > BEAT_HISTORY_LEN {
            self.energy_history.pop_front();
        }
    }

    fn update_auto_mode(&mut self) {
        if self.mode != ViewMode::Auto {
            self.auto_switch_cooldown = self.auto_switch_cooldown.saturating_sub(1);
            return;
        }

        let target = self.choose_auto_mode();
        let should_switch =
            target != self.auto_mode && (self.auto_switch_cooldown == 0 || self.beat);

        if should_switch {
            self.auto_mode = target;
            self.auto_switch_cooldown = AUTO_SWITCH_COOLDOWN_FRAMES;
        } else {
            self.auto_switch_cooldown = self.auto_switch_cooldown.saturating_sub(1);
        }
    }

    fn choose_auto_mode(&self) -> ViewMode {
        if self.beat {
            return if self.beat_intensity > 0.45 {
                ViewMode::Particles
            } else {
                ViewMode::Pulse
            };
        }

        if self.rms < 0.03 || self.sound_type == SoundType::Silence {
            return ViewMode::Spinner;
        }

        match self.sound_type {
            SoundType::Voice => {
                if self.mid > self.bass * 1.2 {
                    ViewMode::Wave
                } else {
                    ViewMode::Pulse
                }
            }
            SoundType::Noise => ViewMode::Rain,
            SoundType::Music => {
                if self.treble > self.bass * 1.15 {
                    ViewMode::Spectrogram
                } else {
                    ViewMode::Bars
                }
            }
            SoundType::Silence => ViewMode::Spinner,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{App, SoundType, Theme, ViewMode, BEAT_COOLDOWN_FRAMES};

    #[test]
    fn detect_beat_triggers_on_energy_spikes() {
        let mut app = App::new(ViewMode::Bars, 1.0, false, false, Theme::Neon);
        for _ in 0..20 {
            app.detect_beat(0.08);
        }

        app.detect_beat(0.28);

        assert!(app.beat);
        assert!(app.beat_intensity > 0.0);
    }

    #[test]
    fn detect_beat_uses_cooldown_between_spikes() {
        let mut app = App::new(ViewMode::Bars, 1.0, false, false, Theme::Neon);
        for _ in 0..20 {
            app.detect_beat(0.08);
        }

        app.detect_beat(0.28);
        assert_eq!(app.beat_cooldown, BEAT_COOLDOWN_FRAMES);

        app.detect_beat(0.28);
        assert!(!app.beat);
    }

    #[test]
    fn active_mode_uses_auto_selection_only_in_auto_mode() {
        let mut app = App::new(ViewMode::Bars, 1.0, false, false, Theme::Neon);
        app.auto_mode = ViewMode::Particles;
        assert_eq!(app.active_mode(), ViewMode::Bars);

        app.set_mode(ViewMode::Auto);
        assert_eq!(app.active_mode(), ViewMode::Particles);
    }

    #[test]
    fn toggle_auto_manual_returns_to_last_manual_mode() {
        let mut app = App::new(ViewMode::Bars, 1.0, false, false, Theme::Neon);
        app.set_mode(ViewMode::Spectrogram);

        app.toggle_auto_manual();
        assert_eq!(app.mode, ViewMode::Auto);

        app.toggle_auto_manual();
        assert_eq!(app.mode, ViewMode::Spectrogram);
    }

    #[test]
    fn numeric_modes_update_last_manual_mode() {
        let mut app = App::new(ViewMode::Auto, 1.0, false, false, Theme::Neon);

        app.set_mode(ViewMode::Rain);
        app.toggle_auto_manual();
        app.toggle_auto_manual();

        assert_eq!(app.mode, ViewMode::Rain);
    }

    #[test]
    fn auto_mode_selects_spinner_for_silence() {
        let mut app = App::new(ViewMode::Auto, 1.0, false, false, Theme::Neon);
        app.rms = 0.0;
        app.sound_type = SoundType::Silence;

        app.update_auto_mode();

        assert_eq!(app.auto_mode, ViewMode::Spinner);
    }

    #[test]
    fn auto_mode_selects_particles_for_strong_beats() {
        let mut app = App::new(ViewMode::Auto, 1.0, false, false, Theme::Neon);
        app.rms = 0.8;
        app.beat = true;
        app.beat_intensity = 0.9;

        app.update_auto_mode();

        assert_eq!(app.auto_mode, ViewMode::Particles);
    }

    #[test]
    fn auto_mode_selects_bars_for_bass_heavy_music() {
        let mut app = App::new(ViewMode::Auto, 1.0, false, false, Theme::Neon);
        app.rms = 0.3;
        app.bass = 0.6;
        app.treble = 0.2;
        app.sound_type = SoundType::Music;

        app.update_auto_mode();

        assert_eq!(app.auto_mode, ViewMode::Bars);
    }
}

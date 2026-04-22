pub mod bars;
pub mod rain;
pub mod wave;
pub mod pulse;
pub mod spectrogram;
pub mod spinner;
pub mod particles;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Wave,
    Bars,
    Rain,
    Pulse,
    Spectrogram,
    Spinner,
    Particles,
}

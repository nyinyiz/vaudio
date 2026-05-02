use clap::ValueEnum;

pub mod bars;
pub mod particles;
pub mod pulse;
pub mod rain;
pub mod spectrogram;
pub mod spinner;
pub mod wave;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ViewMode {
    Auto,
    Wave,
    Bars,
    Rain,
    Pulse,
    Spectrogram,
    Spinner,
    Particles,
}

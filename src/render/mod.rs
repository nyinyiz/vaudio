pub mod bars;
pub mod rain;
pub mod wave;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Wave,
    Bars,
    Rain,
}

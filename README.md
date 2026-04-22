# vaudio

A real-time microphone-driven terminal visualizer built with Rust.

## Features
- **Real-time Visualization**: High-performance audio processing with FFT.
- **Three Modes**:
  - `bars`: Frequency spectrum equalizer with peak-hold effect.
  - `wave`: Scrolling raw waveform.
  - `rain`: Matrix-inspired falling rain modulated by audio intensity.
- **Customizable**: Control FPS, sensitivity, and input device via CLI.
- **Interactive**: Switch modes and adjust sensitivity on the fly.

## Installation
Ensure you have Rust and Cargo installed.

```bash
cargo build --release
```

## Usage
Run with default settings:
```bash
cargo run --release
```

### CLI Options
- `--mode <bars|wave|rain>`: Set initial visualization mode.
- `--fps <number>`: Target frames per second (default: 30).
- `--sensitivity <number>`: Audio sensitivity multiplier (default: 1.0).
- `--device <name>`: Specify input device name or index.
- `--list`: List all available input devices.
- `--no-color`: Disable color output.
- `--mirror`: Mirror the visualization (bars mode).

### Keyboard Controls
- `q`: Quit
- `1`: Switch to **Wave** mode
- `2`: Switch to **Bars** mode
- `3`: Switch to **Rain** mode
- `+`: Increase sensitivity
- `-`: Decrease sensitivity

## Tech Stack
- **ratatui**: Terminal UI framework.
- **cpal**: Cross-platform audio capture.
- **rustfft**: High-performance FFT library.
- **crossterm**: Terminal backend and input handling.
- **clap**: Command-line argument parsing.

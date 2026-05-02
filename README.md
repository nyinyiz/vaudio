# vaudio

A polished, real-time microphone-driven terminal visualizer built with Rust.

## Gallery

| Bars Mode | Wave Mode | Rain Mode |
| :---: | :---: | :---: |
| ![Bars](./ss/bars.png) | ![Wave](./ss/wave.png) | ![Rain](./ss/rain.png) |

| Pulse Mode | Spectrogram | Spinner |
| :---: | :---: | :---: |
| ![Pulse](./ss/pulse.png) | ![Spectrogram](./ss/spectrogram.png) | ![Spinner](./ss/spinner.png) |

| Particles Mode |
| :---: |
| ![Particles](./ss/particles.png) |

**Pro Tip:** Use [VHS](https://github.com/charmbracelet/vhs) to create beautiful animated GIFs of `vaudio` in action!

## Quick Start

If you have Rust installed, you can build and run `vaudio` immediately:

```bash
# Clone the repository
git clone https://github.com/nyinyiz/vaudio.git
cd vaudio

# Build and run in one step
cargo run --release
```

## Installation & Building

### Prerequisites
- **Rust Stable**: [Install Rust](https://rustup.rs/)
- **System Dependencies**:
  - **macOS**: None (uses native CoreAudio).
  - **Linux**: Requires `libasound2-dev` (ALSA).
  - **Windows**: None (uses WASAPI).

### Build for Release
To build a highly optimized binary:
```bash
cargo build --release
```
The resulting binary will be located at: `./target/release/vaudio`.

### Global Installation
To install `vaudio` as a command on your system:
```bash
cargo install --path .
```
After this, you can simply run `vaudio` from any directory.

## Usage

```bash
vaudio [OPTIONS]
```

### CLI Options
- `--mode <auto|bars|wave|rain|pulse|spectrogram|spinner|particles>`: Set initial visualization mode (default: `bars`).
- `--fps <number>`: Target frames per second (default: `30`).
- `--sensitivity <number>`: Audio sensitivity multiplier (default: `10.0`).
- `--device <name>`: Specify input device name or index.
- `--list`: List all available input devices.
- `--no-color`: Disable color output for a monochrome look.
- `--mirror`: Mirror the visualization (especially useful in `bars` mode).
- `--theme <neon|fire|ice|rainbow>`: Set the color theme (default: `neon`).

### Keyboard Controls
- `q`: Quit
- `1`: Switch to **Wave** mode
- `2`: Switch to **Bars** mode
- `3`: Switch to **Rain** mode
- `4`: Switch to **Pulse** mode
- `5`: Switch to **Spectrogram** mode
- `6`: Switch to **Spinner** mode
- `7`: Switch to **Particles** mode
- `8`: Switch to **Auto** mode
- `m`: Toggle between **Auto** and the last manual mode
- `t`: Cycle color theme
- `+`: Increase sensitivity
- `-`: Decrease sensitivity

## Visual Modes
1. **Auto**: Selects the best visualizer from live audio state.
2. **Bars**: A classic frequency spectrum equalizer with peak-hold decay.
3. **Wave**: A scrolling real-time waveform of the raw audio signal.
4. **Rain**: A "Matrix" style falling character effect.
5. **Pulse**: Concentric expanding shockwaves driven by audio volume.
6. **Spectrogram**: A waterfall heatmap of frequency history scrolling downwards.
7. **Spinner**: A rotating starburst that speeds up and expands with the beat.
8. **Particles**: A fireworks-like explosion of characters flying from the center.

### Auto Mode
Auto mode watches the live signal classification, beat detector, and bass/mid/treble levels to select an active visualizer:

- **Silence / low input**: Spinner
- **Voice-heavy input**: Wave or Pulse
- **Music / bass-heavy input**: Bars
- **Treble-heavy music**: Spectrogram
- **Noise / wide-band input**: Rain
- **Beat spikes**: Pulse or Particles

The HUD shows both the selected mode and Auto's current target, for example `AUTO > BARS`.

## Terminal Previews
These are terminal mock screenshots showing the intended output format. They are not captured from a specific microphone session, but they match the current layout and HUD structure.

### Auto Mode: Music
```text
████████████████████████████████████████████████████████████████████████████████
████████████████████████████████████████████████████████████████████████████████
▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
  AUTO > BARS  NEON        VOL [########--]  B [####-] M [###--] T [##---]     MUSIC  BEAT  SENS 8.4
                  [1-8] modes   [t] theme   [m] auto/manual   [+/-] sensitivity   [q] quit
```

### Auto Mode: Beat Burst
```text
                  *             +             *                  +
        +                 *             *              + 
             .      *          +     *       +      .          *
                     +       *     *     *       +
                 *        +      *   *      +        *
        .             +       *       *       +             .
  AUTO > PARTICLES  FIRE     VOL [#########-]  B [#####] M [###--] T [##---]     MUSIC  BEAT  SENS 9.2
                  [1-8] modes   [t] theme   [m] auto/manual   [+/-] sensitivity   [q] quit
```

### Auto Mode: Voice
```text
────────────────────────────────────────────────────────────────────────────────
                 ┃┃┃┃┃       ┃┃┃┃┃┃          ┃┃┃┃
             ┃┃┃┃┃┃┃┃┃┃┃  ┃┃┃┃┃┃┃┃┃┃┃    ┃┃┃┃┃┃┃┃┃
         ┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃
             ┃┃┃┃┃┃┃┃┃┃┃  ┃┃┃┃┃┃┃┃┃┃┃    ┃┃┃┃┃┃┃┃┃
                 ┃┃┃┃┃       ┃┃┃┃┃┃          ┃┃┃┃
  AUTO > WAVE  ICE          VOL [#####-----]  B [#----] M [####-] T [##---]     VOICE  ----  SENS 7.6
                  [1-8] modes   [t] theme   [m] auto/manual   [+/-] sensitivity   [q] quit
```

## Tech Stack
- **ratatui**: A powerful library for building terminal user interfaces.
- **cpal**: Cross-platform audio capture.
- **rustfft**: High-performance Fast Fourier Transform library.
- **crossterm**: Terminal backend and cross-platform input handling.
- **clap**: Robust command-line argument parsing.

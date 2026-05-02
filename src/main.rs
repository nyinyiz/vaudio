mod app;
mod audio;
mod render;
mod signal;
mod theme;

use anyhow::Result;
use app::App;
use audio::AudioCapture;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Terminal,
};
use render::{
    bars::BarsWidget, particles::ParticlesWidget, pulse::PulseWidget, rain::RainWidget,
    spectrogram::SpectrogramWidget, spinner::SpinnerWidget, wave::WaveWidget, ViewMode,
};
use std::{
    io,
    sync::mpsc,
    time::{Duration, Instant},
};
use theme::Theme;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Visualization mode
    #[arg(short, long, value_enum, default_value_t = ViewMode::Bars)]
    mode: ViewMode,

    /// Target frames per second
    #[arg(short, long, default_value_t = 30)]
    fps: u32,

    /// Input sensitivity multiplier (Max: 10.0)
    #[arg(short, long, default_value_t = 10.0)]
    sensitivity: f32,

    /// Device name or index (use --list to see options)
    #[arg(short, long)]
    device: Option<String>,

    /// List available input devices
    #[arg(short, long)]
    list: bool,

    /// Disable colors
    #[arg(long)]
    no_color: bool,

    /// Color theme
    #[arg(long, value_enum, default_value_t = Theme::Neon)]
    theme: Theme,

    /// Mirror the visualization
    #[arg(long)]
    mirror: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.list {
        let devices = audio::list_devices()?;
        for d in devices {
            println!("{}", d);
        }
        return Ok(());
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Audio setup
    let (tx, rx) = mpsc::sync_channel(10);
    let _capture = AudioCapture::new(args.device, tx)?;

    let mut app = App::new(
        args.mode,
        args.sensitivity,
        args.mirror,
        args.no_color,
        args.theme,
    );
    let tick_rate = Duration::from_secs_f32(1.0 / args.fps as f32);

    // Splash screen
    let splash_start = Instant::now();
    while splash_start.elapsed() < Duration::from_secs(1) {
        terminal.draw(|f| {
            let area = f.size();
            let text = "vaudio";
            let x = (area.width as i16 - text.len() as i16) / 2;
            let y = area.height / 2;
            if x >= 0 {
                f.buffer_mut().set_string(
                    x as u16,
                    y,
                    text,
                    ratatui::style::Style::default().fg(Color::Green),
                );
            }
        })?;
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(_) = event::read()? {
                break;
            }
        }
    }

    let res = run_app(&mut terminal, &mut app, rx, tick_rate);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

impl std::fmt::Display for ViewMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mode = match self {
            ViewMode::Wave => "wave",
            ViewMode::Bars => "bars",
            ViewMode::Rain => "rain",
            ViewMode::Pulse => "pulse",
            ViewMode::Spectrogram => "spectrogram",
            ViewMode::Spinner => "spinner",
            ViewMode::Particles => "particles",
        };
        f.write_str(mode)
    }
}

#[cfg(test)]
mod tests {
    use super::{Args, Parser, ViewMode};

    #[test]
    fn parses_valid_mode_names() {
        let args = Args::try_parse_from(["vaudio", "--mode", "spectrogram"]).unwrap();

        assert_eq!(args.mode, ViewMode::Spectrogram);
    }

    #[test]
    fn rejects_invalid_mode_names() {
        let err = Args::try_parse_from(["vaudio", "--mode", "unknown"]).unwrap_err();

        assert_eq!(err.kind(), clap::error::ErrorKind::InvalidValue);
    }

    #[test]
    fn parses_theme_names() {
        let args = Args::try_parse_from(["vaudio", "--theme", "fire"]).unwrap();

        assert_eq!(args.theme, crate::theme::Theme::Fire);
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rx: mpsc::Receiver<Vec<f32>>,
    tick_rate: Duration,
) -> Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('1') => app.set_mode(ViewMode::Wave),
                    KeyCode::Char('2') => app.set_mode(ViewMode::Bars),
                    KeyCode::Char('3') => app.set_mode(ViewMode::Rain),
                    KeyCode::Char('4') => app.set_mode(ViewMode::Pulse),
                    KeyCode::Char('5') => app.set_mode(ViewMode::Spectrogram),
                    KeyCode::Char('6') => app.set_mode(ViewMode::Spinner),
                    KeyCode::Char('7') => app.set_mode(ViewMode::Particles),
                    KeyCode::Char('+') => app.adjust_sensitivity(0.2),
                    KeyCode::Char('-') => app.adjust_sensitivity(-0.2),
                    KeyCode::Char('t') => app.cycle_theme(),
                    _ => {}
                }
            }
        }

        // Process all available audio chunks
        let size = terminal.size()?;
        while let Ok(samples) = rx.try_recv() {
            app.update_audio(&samples, size.width, size.height);
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.size());

    let area = chunks[0];
    let help_area = chunks[1];

    let palette = app.theme.palette(app.no_color);

    // Render the visualizer in the main area
    match app.mode {
        ViewMode::Bars => {
            f.render_widget(
                BarsWidget {
                    data: &app.fft_data,
                    peaks: &app.peaks,
                    peak_color: palette.peak,
                    levels: palette.levels,
                    mirror: app.mirror,
                },
                area,
            );
        }
        ViewMode::Wave => {
            f.render_widget(
                WaveWidget {
                    samples: &app.wave_data,
                    accent_color: palette.accent,
                    levels: palette.levels,
                },
                area,
            );
        }
        ViewMode::Rain => {
            f.render_widget(
                RainWidget {
                    drops: &app.rain_drops,
                    peak_color: palette.peak,
                    levels: palette.levels,
                },
                area,
            );
        }
        ViewMode::Pulse => {
            f.render_widget(
                PulseWidget {
                    rings: &app.pulse_rings,
                    levels: palette.levels,
                },
                area,
            );
        }
        ViewMode::Spectrogram => {
            f.render_widget(
                SpectrogramWidget {
                    history: &app.spectrogram_history,
                    levels: palette.levels,
                },
                area,
            );
        }
        ViewMode::Spinner => {
            f.render_widget(
                SpinnerWidget {
                    angle: app.spinner_angle,
                    rms: app.rms,
                    levels: palette.levels,
                },
                area,
            );
        }
        ViewMode::Particles => {
            f.render_widget(
                ParticlesWidget {
                    particles: &app.particles,
                    levels: palette.levels,
                },
                area,
            );
        }
    }

    // Render the help bar at the bottom
    let mode_str = format!("{:?}", app.mode).to_uppercase();
    let theme_str = format!("{:?}", app.theme).to_uppercase();
    let sound_str = format!("{:?}", app.sound_type).to_uppercase();
    let beat_str = if app.beat { "YES" } else { "NO" };
    let help_text = vec![Line::from(vec![
        Span::styled(" MODE: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(mode_str, Style::default().fg(palette.peak)),
        Span::raw(" | "),
        Span::styled(" THEME: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(theme_str, Style::default().fg(palette.accent)),
        Span::raw(" | "),
        Span::styled(" DETECTED: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(sound_str, Style::default().fg(palette.accent)),
        Span::raw(" | "),
        Span::styled(" BEAT: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(beat_str, Style::default().fg(palette.primary)),
        Span::raw(" | "),
        Span::styled(" SENS: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("{:.1}", app.sensitivity),
            Style::default().fg(palette.peak),
        ),
        Span::raw(" | "),
        Span::styled(" KEYS: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("[1-7] Modes [t] Theme [+/-] Sens [q] Quit"),
    ])];

    let help_bar =
        Paragraph::new(help_text).style(Style::default().bg(palette.help_bg).fg(palette.help_fg));
    f.render_widget(help_bar, help_area);
}

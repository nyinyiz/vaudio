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
    layout::{Alignment, Constraint, Direction, Layout},
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
            ViewMode::Auto => "auto",
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
    fn parses_auto_mode() {
        let args = Args::try_parse_from(["vaudio", "--mode", "auto"]).unwrap();

        assert_eq!(args.mode, ViewMode::Auto);
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
                    KeyCode::Char('8') => app.set_mode(ViewMode::Auto),
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
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(f.size());

    let area = chunks[0];
    let help_area = chunks[1];

    let palette = app.theme.palette(app.no_color);

    // Render the visualizer in the main area
    match app.active_mode() {
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
        ViewMode::Auto | ViewMode::Spinner => {
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

    // Render the HUD at the bottom
    let hud_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(help_area);
    let hud_top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(hud_rows[0]);

    let mode_str = mode_label(app);
    let theme_str = format!("{:?}", app.theme).to_uppercase();
    let sound_str = format!("{:?}", app.sound_type).to_uppercase();
    let beat_str = if app.beat { "BEAT" } else { "----" };
    let hud_style = Style::default().bg(palette.help_bg).fg(palette.help_fg);

    let identity = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled(mode_str, Style::default().fg(palette.peak)),
        Span::raw("  "),
        Span::styled(theme_str, Style::default().fg(palette.accent)),
    ]))
    .style(hud_style);
    f.render_widget(identity, hud_top[0]);

    let levels = Paragraph::new(Line::from(vec![
        Span::styled("VOL ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            level_meter(app.rms, 10),
            Style::default().fg(palette.level(app.rms)),
        ),
        Span::raw("  "),
        Span::styled("B ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            level_meter(app.bass, 5),
            Style::default().fg(palette.level(app.bass)),
        ),
        Span::raw(" "),
        Span::styled("M ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            level_meter(app.mid, 5),
            Style::default().fg(palette.level(app.mid)),
        ),
        Span::raw(" "),
        Span::styled("T ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            level_meter(app.treble, 5),
            Style::default().fg(palette.level(app.treble)),
        ),
    ]))
    .alignment(Alignment::Center)
    .style(hud_style);
    f.render_widget(levels, hud_top[1]);

    let detection = Paragraph::new(Line::from(vec![
        Span::styled(sound_str, Style::default().fg(palette.accent)),
        Span::raw("  "),
        Span::styled(beat_str, Style::default().fg(palette.primary)),
        Span::raw("  "),
        Span::styled("SENS ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("{:.1}", app.sensitivity),
            Style::default().fg(palette.peak),
        ),
        Span::raw(" "),
    ]))
    .alignment(Alignment::Right)
    .style(hud_style);
    f.render_widget(detection, hud_top[2]);

    let controls = Paragraph::new(Line::from(vec![
        Span::styled("[1-8]", Style::default().fg(palette.peak)),
        Span::raw(" modes   "),
        Span::styled("[t]", Style::default().fg(palette.peak)),
        Span::raw(" theme   "),
        Span::styled("[+/-]", Style::default().fg(palette.peak)),
        Span::raw(" sensitivity   "),
        Span::styled("[q]", Style::default().fg(palette.peak)),
        Span::raw(" quit"),
    ]))
    .alignment(Alignment::Center)
    .style(hud_style);
    f.render_widget(controls, hud_rows[1]);
}

fn level_meter(value: f32, width: usize) -> String {
    let filled = (value.clamp(0.0, 1.0) * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "#".repeat(filled), "-".repeat(empty))
}

fn mode_label(app: &App) -> String {
    if app.mode == ViewMode::Auto {
        format!("AUTO > {:?}", app.auto_mode).to_uppercase()
    } else {
        format!("{:?}", app.mode).to_uppercase()
    }
}

#[cfg(test)]
mod hud_tests {
    use super::{level_meter, mode_label, App, Theme, ViewMode};

    #[test]
    fn level_meter_clamps_values() {
        assert_eq!(level_meter(-1.0, 4), "[----]");
        assert_eq!(level_meter(2.0, 4), "[####]");
    }

    #[test]
    fn level_meter_renders_partial_values() {
        assert_eq!(level_meter(0.5, 6), "[###---]");
    }

    #[test]
    fn mode_label_shows_auto_target() {
        let mut app = App::new(ViewMode::Auto, 1.0, false, false, Theme::Neon);
        app.auto_mode = ViewMode::Spectrogram;

        assert_eq!(mode_label(&app), "AUTO > SPECTROGRAM");
    }
}

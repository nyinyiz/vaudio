mod app;
mod audio;
mod render;
mod signal;

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
    style::Color,
    Terminal,
};
use render::{bars::BarsWidget, rain::RainWidget, wave::WaveWidget, ViewMode};
use std::{
    io,
    sync::mpsc,
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Visualization mode: wave, bars, rain
    #[arg(short, long, default_value = "bars")]
    mode: String,

    /// Target frames per second
    #[arg(short, long, default_value_t = 30)]
    fps: u32,

    /// Input sensitivity multiplier
    #[arg(short, long, default_value_t = 1.0)]
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

    /// Mirror the visualization
    #[arg(short, long)]
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

    let mode = match args.mode.as_str() {
        "wave" => ViewMode::Wave,
        "bars" => ViewMode::Bars,
        "rain" => ViewMode::Rain,
        _ => ViewMode::Bars,
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Audio setup
    let (tx, rx) = mpsc::sync_channel(10);
    let _capture = AudioCapture::new(args.device, tx)?;

    let mut app = App::new(mode, args.sensitivity, args.mirror, args.no_color);
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
                f.buffer_mut().set_string(x as u16, y, text, ratatui::style::Style::default().fg(Color::Green));
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
                    KeyCode::Char('+') => app.adjust_sensitivity(0.2),
                    KeyCode::Char('-') => app.adjust_sensitivity(-0.2),
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
    let area = f.size();
    let color = if app.no_color { Color::White } else { Color::Green };

    match app.mode {
        ViewMode::Bars => {
            let widget = BarsWidget {
                data: &app.fft_data,
                peaks: &app.peaks,
                color,
                mirror: app.mirror,
            };
            f.render_widget(widget, area);
        }
        ViewMode::Wave => {
            let widget = WaveWidget {
                samples: &app.wave_data,
                color,
            };
            f.render_widget(widget, area);
        }
        ViewMode::Rain => {
            let widget = RainWidget {
                drops: &app.rain_drops,
                color,
                intensity: app.rms,
            };
            f.render_widget(widget, area);
        }
    }
}

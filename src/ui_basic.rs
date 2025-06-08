use crate::keylogger::KeyStats;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{io, sync::mpsc, time::Duration};
use tokio::sync::watch;

pub struct App {
    pub should_quit: bool,
    pub stats: KeyStats,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            stats: KeyStats::new(),
        }
    }
}

pub fn run_ui(
    stats_rx: mpsc::Receiver<KeyStats>,
    shutdown_tx: watch::Sender<bool>,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| render_basic_ui(f, &app))?;

        if crossterm::event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    _ => {}
                }
            }
        }

        // Update stats
        while let Ok(stats) = stats_rx.try_recv() {
            app.stats = stats;
        }

        if app.should_quit {
            let _ = shutdown_tx.send(true);
            break;
        }
    }

    disable_raw_mode()?;
    Ok(())
}

fn render_basic_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)].as_ref())
        .split(f.size());

    let info = Paragraph::new(format!(
        "📊 CtrlQ Keylogger\n\n\
         Total Keystrokes: {}\n\
         WPM: {:.1}\n\n\
         Press 'q' to quit",
        app.stats.total_keystrokes,
        app.stats.get_wpm().unwrap_or(0.0),
    ))
    .block(Block::default().borders(Borders::ALL).title("Statistics"))
    .style(Style::default().fg(Color::White));

    f.render_widget(info, chunks[0]);
}

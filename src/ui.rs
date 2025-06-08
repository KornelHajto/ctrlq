use crate::keylogger::KeyStats;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Gauge, List, ListItem, Paragraph, Tabs, Wrap,
    },
    Frame, Terminal,
};
use std::{
    io,
    sync::mpsc,
    time::{Duration, Instant},
};
use tokio::sync::watch;

pub struct App {
    pub should_quit: bool,
    pub selected_tab: usize,
    pub stats: KeyStats,
    pub last_update: Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            selected_tab: 0,
            stats: KeyStats::new(),
            last_update: Instant::now(),
        }
    }

    pub fn update_stats(&mut self, stats: KeyStats) {
        self.stats = stats;
        self.last_update = Instant::now();
    }
}

pub fn run_ui(
    stats_rx: mpsc::Receiver<KeyStats>,
    shutdown_tx: watch::Sender<bool>,
    reset_tx: mpsc::Sender<()>,
) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Run the app
    let res = run_app(&mut terminal, &mut app, stats_rx, shutdown_tx, reset_tx);

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

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    stats_rx: mpsc::Receiver<KeyStats>,
    shutdown_tx: watch::Sender<bool>,
    reset_tx: mpsc::Sender<()>,
) -> Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        // Draw UI
        terminal.draw(|f| ui(f, app))?;

        // Handle events
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    KeyCode::Tab => {
                        app.selected_tab = (app.selected_tab + 1) % 4;
                    }
                    KeyCode::Char('r') => {
                        // Reset stats - send signal to keylogger thread
                        if reset_tx.send(()).is_ok() {
                            app.stats = KeyStats::new();
                        }
                    }
                    _ => {}
                }
            }
        }

        // Update stats
        while let Ok(stats) = stats_rx.try_recv() {
            app.update_stats(stats);
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.should_quit {
            let _ = shutdown_tx.send(true);
            break;
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(f.area());

    // Header with tabs
    let titles: Vec<Line> = vec!["Overview", "Top Keys", "Heatmap", "Sessions"]
        .iter()
        .cloned()
        .map(Line::from)
        .collect();
    
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("CtrlQ - Developer Keylogger"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(app.selected_tab);
    
    f.render_widget(tabs, chunks[0]);

    // Main content based on selected tab
    match app.selected_tab {
        0 => render_overview(f, chunks[1], app),
        1 => render_top_keys(f, chunks[1], app),
        2 => render_heatmap(f, chunks[1], app),
        3 => render_sessions(f, chunks[1], app),
        _ => {}
    }
}

fn render_overview(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(10),
            Constraint::Min(0),
        ])
        .split(area);

    // Stats summary
    let total_keys = app.stats.total_keystrokes;
    let wpm = app.stats.get_wpm().unwrap_or(0.0);
    let session_duration = chrono::Utc::now()
        .signed_duration_since(app.stats.session_start)
        .num_minutes();

    let summary = Paragraph::new(format!(
        "📊 Session Stats\n\
         Total Keystrokes: {}\n\
         Session Duration: {} minutes\n\
         Typing Speed: {:.1} WPM\n\
         Unique Keys: {}",
        total_keys,
        session_duration,
        wpm,
        app.stats.key_counts.len()
    ))
    .block(Block::default().borders(Borders::ALL).title("Summary"))
    .style(Style::default().fg(Color::Green));

    f.render_widget(summary, chunks[0]);

    // Recent key sequence
    let recent_keys: String = app.stats.key_sequences
        .iter()
        .rev()
        .take(20)
        .map(|k| {
            // Simplify key names for display
            match k.as_str() {
                "KEY_SPACE" => " ",
                "KEY_ENTER" => "↵",
                "KEY_TAB" => "⭾",
                "KEY_BACKSPACE" => "⌫",
                k if k.starts_with("KEY_") => &k[4..],
                _ => k,
            }
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" ");

    let recent = Paragraph::new(recent_keys)
        .block(Block::default().borders(Borders::ALL).title("Recent Keys"))
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::Cyan));

    f.render_widget(recent, chunks[1]);

    // Activity gauge
    let activity_level = if total_keys > 0 { 
        std::cmp::min(100, (total_keys as f64 / 10.0) as u16)
    } else { 
        0 
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Activity Level"))
        .gauge_style(Style::default().fg(Color::Magenta))
        .percent(activity_level)
        .label(format!("{}%", activity_level));

    f.render_widget(gauge, chunks[2]);
}

fn render_top_keys(f: &mut Frame, area: Rect, app: &App) {
    let top_keys = app.stats.get_top_keys(20);
    
    let items: Vec<ListItem> = top_keys
        .iter()
        .enumerate()
        .map(|(i, (key, count))| {
            let key_display = match key.as_str() {
                "KEY_SPACE" => "SPACE".to_string(),
                "KEY_ENTER" => "ENTER".to_string(),
                "KEY_TAB" => "TAB".to_string(),
                "KEY_BACKSPACE" => "BACKSPACE".to_string(),
                k if k.starts_with("KEY_") => k[4..].to_string(),
                _ => key.clone(),
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:2}. ", i + 1),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("{:<15}", key_display),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:>8}", count),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!(" ({:.1}%)", *count as f64 / app.stats.total_keystrokes as f64 * 100.0),
                    Style::default().fg(Color::Gray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Top Keys"))
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area);
}

fn render_heatmap(f: &mut Frame, area: Rect, app: &App) {
    // Linux input key code to character mapping for QWERTY layout
    let key_layout = [
        // Row 1 (numbers)
        ("1234567890", vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
        // Row 2 (QWERTY)
        ("QWERTYUIOP", vec![16, 17, 18, 19, 20, 21, 22, 23, 24, 25]),
        // Row 3 (ASDF)
        ("ASDFGHJKL", vec![30, 31, 32, 33, 34, 35, 36, 37, 38]),
        // Row 4 (ZXCV)
        ("ZXCVBNM", vec![44, 45, 46, 47, 48, 49, 50]),
    ];

    let mut heatmap_text = Vec::new();
    let max_count = app.stats.key_counts.values().max().copied().unwrap_or(1);

    for (chars, codes) in key_layout.iter() {
        let mut line_spans = Vec::new();
        for (i, ch) in chars.chars().enumerate() {
            if let Some(&key_code) = codes.get(i) {
                let key_name = format!("KEY_{}", key_code);
                let count = app.stats.key_counts.get(&key_name).copied().unwrap_or(0);
                let intensity = if max_count > 0 { count as f64 / max_count as f64 } else { 0.0 };
                
                let color = if intensity > 0.8 {
                    Color::Red
                } else if intensity > 0.6 {
                    Color::LightRed
                } else if intensity > 0.4 {
                    Color::Yellow
                } else if intensity > 0.2 {
                    Color::Green
                } else if intensity > 0.0 {
                    Color::Blue
                } else {
                    Color::Gray
                };

                let display_char = if count > 0 {
                    format!("{}[{}] ", ch, count)
                } else {
                    format!("{} ", ch)
                };

                line_spans.push(Span::styled(
                    display_char,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ));
            }
        }
        heatmap_text.push(Line::from(line_spans));
        heatmap_text.push(Line::from(""));
    }

    // Add space bar
    let space_count = app.stats.key_counts.get("KEY_57").copied().unwrap_or(0);
    let space_intensity = if max_count > 0 { space_count as f64 / max_count as f64 } else { 0.0 };
    let space_color = if space_intensity > 0.8 {
        Color::Red
    } else if space_intensity > 0.6 {
        Color::LightRed
    } else if space_intensity > 0.4 {
        Color::Yellow
    } else if space_intensity > 0.2 {
        Color::Green
    } else if space_intensity > 0.0 {
        Color::Blue
    } else {
        Color::Gray
    };

    heatmap_text.push(Line::from(vec![
        Span::styled(
            format!("      SPACE[{}]      ", space_count),
            Style::default().fg(space_color).add_modifier(Modifier::BOLD),
        )
    ]));
    heatmap_text.push(Line::from(""));

    // Add legend
    heatmap_text.push(Line::from(vec![
        Span::styled("Legend: ", Style::default().fg(Color::White)),
        Span::styled("■ Very High ", Style::default().fg(Color::Red)),
        Span::styled("■ High ", Style::default().fg(Color::LightRed)),
        Span::styled("■ Medium ", Style::default().fg(Color::Yellow)),
        Span::styled("■ Low ", Style::default().fg(Color::Green)),
        Span::styled("■ Very Low ", Style::default().fg(Color::Blue)),
        Span::styled("■ Unused ", Style::default().fg(Color::Gray)),
    ]));

    let heatmap = Paragraph::new(Text::from(heatmap_text))
        .block(Block::default().borders(Borders::ALL).title("Keyboard Heatmap"))
        .alignment(Alignment::Center);

    f.render_widget(heatmap, area);
}

fn render_sessions(f: &mut Frame, area: Rect, app: &App) {
    let info = Paragraph::new(format!(
        "📈 Session Information\n\n\
         Current Session Started: {}\n\
         Total Sessions: {}\n\
         Average WPM: {:.1}\n\n\
         Daily Statistics:\n\
         Today's Keystrokes: {}\n\n\
         Press 'q' to quit, 'Tab' to switch tabs, 'r' to reset stats",
        app.stats.session_start.format("%Y-%m-%d %H:%M:%S"),
        app.stats.typing_sessions.len() + 1,
        app.stats.get_wpm().unwrap_or(0.0),
        app.stats.total_keystrokes,
    ))
    .block(Block::default().borders(Borders::ALL).title("Session Info"))
    .wrap(Wrap { trim: true })
    .style(Style::default().fg(Color::White));

    f.render_widget(info, area);
}

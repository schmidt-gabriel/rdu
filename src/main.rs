mod app;
mod scanner;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    env, io,
    time::{Duration, Instant},
};

fn main() -> Result<()> {
    let arg = env::args().nth(1);

    if let Some(a) = &arg {
        if a == "--version" || a == "-V" {
            println!("rdu {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        } else if a == "--help" || a == "-h" {
            println!("rdu - Rust Disk Usage\n");
            println!("Usage: rdu [PATH]\n");
            println!("Arguments:");
            println!("  PATH             Directory to scan (defaults to \".\")\n");
            println!("Options:");
            println!("  -h, --help       Print help");
            println!("  -V, --version    Print version");
            return Ok(());
        }
    }

    let root_path = arg.unwrap_or_else(|| ".".to_string());

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, root_path);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("Error: {e}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    root_path: String,
) -> Result<()> {
    let mut app = App::new(root_path);

    // Kick off background scan immediately
    app.start_scan();

    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // Poll for new scan results
        app.poll_scan();

        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.show_help {
                        app.show_help = false;
                    } else {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('j') | KeyCode::Down => app.select_next(),
                            KeyCode::Char('k') | KeyCode::Up => app.select_prev(),
                            KeyCode::Enter | KeyCode::Right => app.enter_selected(),
                            KeyCode::Backspace | KeyCode::Left | KeyCode::Esc => app.go_up(),
                            KeyCode::Char('s') => app.cycle_sort_mode(),
                            KeyCode::Char('r') => app.start_scan(),
                            KeyCode::Char('?') => app.toggle_help(),
                            _ => {}
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

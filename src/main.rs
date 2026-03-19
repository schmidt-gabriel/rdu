mod app;
mod scanner;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    env, io,
    time::{Duration, Instant},
};

fn main() -> Result<()> {
    let mut root_path = None;
    let mut no_delete = false;

    for arg in env::args().skip(1) {
        if arg == "--version" || arg == "-V" {
            println!("rdu {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        } else if arg == "--help" || arg == "-h" {
            println!("rdu - Rust Disk Usage\n");
            println!("Usage: rdu [OPTIONS] [PATH]\n");
            println!("Arguments:");
            println!("  PATH             Directory to scan (defaults to \".\")\n");
            println!("Options:");
            println!("  -h, --help       Print help");
            println!("  -V, --version    Print version");
            println!("  --no-delete      Prevent deletions");
            return Ok(());
        } else if arg == "--no-delete" {
            no_delete = true;
        } else if !arg.starts_with('-') && root_path.is_none() {
            root_path = Some(arg);
        }
    }

    let root_path = root_path.unwrap_or_else(|| ".".to_string());

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, root_path, no_delete);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("Error: {e}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    root_path: String,
    no_delete: bool,
) -> Result<()> {
    let mut app = App::new(root_path, no_delete);

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
            let event = event::read()?;

            // Always exit on Ctrl+C
            if let Event::Key(key) = event {
                if key.kind == KeyEventKind::Press
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('c')
                {
                    return Ok(());
                }
            }

            if app.show_delete_confirm {
                if let Event::Key(key) = event {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                app.delete_marked();
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                app.show_delete_confirm = false;
                            }
                            _ => {}
                        }
                    }
                }
            } else if app.show_help {
                if matches!(event, Event::Key(_) | Event::Mouse(_)) {
                    app.show_help = false;
                }
            } else {
                match event {
                    Event::Key(key) => {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('q') => return Ok(()),
                                KeyCode::Char('j') | KeyCode::Down => app.select_next(),
                                KeyCode::Char('k') | KeyCode::Up => app.select_prev(),
                                KeyCode::Enter | KeyCode::Right => app.enter_selected(),
                                KeyCode::Backspace | KeyCode::Left | KeyCode::Esc => app.go_up(),
                                KeyCode::Char('s') => app.cycle_sort_mode(),
                                KeyCode::Char('r') => app.start_scan(),
                                KeyCode::Char('?') => app.toggle_help(),
                                KeyCode::Char(' ') => app.toggle_mark(),
                                KeyCode::Char('d') | KeyCode::Char('D') => app.prompt_delete(),
                                _ => {}
                            }
                        }
                    }
                    Event::Mouse(mouse) => match mouse.kind {
                        MouseEventKind::ScrollDown => app.select_next(),
                        MouseEventKind::ScrollUp => app.select_prev(),
                        MouseEventKind::Down(MouseButton::Left) => {
                            let size = terminal.size()?;
                            app.handle_click(mouse.row, size.height);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

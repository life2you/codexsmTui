mod codex;
mod tui;

use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::Event,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    codex::{default_session_root, scan_sessions},
    tui::{app::App, events::next_event, ui::render},
};

fn main() -> Result<()> {
    if handle_cli_flags() {
        return Ok(());
    }

    let scan_root = default_session_root()?;
    let scan = scan_sessions(&scan_root);
    let mut app = App::new(scan_root, scan);

    let mut terminal = setup_terminal()?;
    let result = run_app(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    result
}

fn handle_cli_flags() -> bool {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--version") | Some("-V") => {
            println!("codexsmTui {}", env!("CARGO_PKG_VERSION"));
            true
        }
        Some("--help") | Some("-h") => {
            println!("codexsmTui {}", env!("CARGO_PKG_VERSION"));
            println!("Usage: codexsmTui");
            println!("       codexsmTui --version");
            println!("       codexsmTui --help");
            println!();
            println!("Launches the interactive TUI for managing local Codex CLI sessions.");
            true
        }
        Some(other) => {
            eprintln!("Unsupported argument: {other}");
            eprintln!("Run `codexsmTui --help` for usage.");
            true
        }
        None => false,
    }
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app))?;
        if app.should_quit {
            break;
        }

        if let Some(event) = next_event(Duration::from_millis(250))? {
            if let Event::Key(key) = event {
                app.handle_key(key);
            }
        }
    }

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

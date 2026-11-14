//! InstallMon TUI - Terminal User Interface powered by Ratatui & Crossterm.

pub mod app;
pub mod ui;

use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use app::App;
use installmon_database::Database;

pub fn run_tui(db: Database) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(db);
    app.refresh()?;

    let res = run_loop(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("TUI Error: {:?}", err);
    }

    Ok(())
}

fn run_loop<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let CEvent::Key(key) = event::read()? {
                // Ignore key release events
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }

                // Global quit
                if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                    || (key.code == KeyCode::Char('q') && !app.showing_help)
                {
                    return Ok(());
                }

                app.handle_key(key.code);
            }
        }
    }
}

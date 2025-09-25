use crate::tui::app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{io, time::Duration};

/// Sets up the terminal, runs the TUI, and restores the terminal state.
pub fn run_tui() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run the main loop
    let jobs = Vec::new(); // Placeholder for actual job data
    let app = App::new(&jobs);
    run_app(&mut terminal, app)?;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// The main application loop.
fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| app.draw(f))?;

        // Poll for an event with a timeout.
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j') => app.next(),
                    KeyCode::Char('k') => app.previous(),
                    _ => {}
                }
            }
        }
    }
}

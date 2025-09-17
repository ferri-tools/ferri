//! TUI logic for `ferri flow show`

use crate::flow::{Pipeline, StepKind};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io::{self, Stdout};

type CrosstermTerminal = Terminal<CrosstermBackend<Stdout>>;

// Main entry point for the TUI
pub fn run_tui(pipeline: &Pipeline) -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    run_app(&mut terminal, pipeline)?;
    restore_terminal(&mut terminal)?;
    Ok(())
}

// Setup the terminal for TUI mode
fn setup_terminal() -> io::Result<CrosstermTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

// Restore the terminal to its original state
fn restore_terminal(terminal: &mut CrosstermTerminal) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

// The main application loop
fn run_app(terminal: &mut CrosstermTerminal, pipeline: &Pipeline) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, pipeline))?;

        if let Event::Key(key) = event::read()? {
            if let KeyCode::Char('q') = key.code {
                return Ok(());
            }
        }
    }
}

// The main UI drawing function
fn ui(frame: &mut Frame, pipeline: &Pipeline) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // For title
            Constraint::Min(0),    // For content
            Constraint::Length(1), // For footer
        ])
        .split(frame.size());

    let title_block = Block::default()
        .title(format!(" Pipeline: {} ", pipeline.name))
        .borders(Borders::TOP);
    frame.render_widget(title_block, main_chunks[0]);

    let footer = Paragraph::new("Press 'q' to quit.").style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, main_chunks[2]);

    // Create a layout for the steps inside the main content area
    let num_steps = pipeline.steps.len();
    let constraints: Vec<Constraint> = std::iter::repeat(Constraint::Length(5))
        .take(num_steps)
        .collect();

    let step_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(main_chunks[1]);

    for (i, step) in pipeline.steps.iter().enumerate() {
        let (step_type, color) = match &step.kind {
            StepKind::Model(_) => ("Model", Color::Cyan),
            StepKind::Process(_) => ("Process", Color::Green),
        };

        let step_block = Block::default()
            .title(format!(" {} ", step.name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color));

        let step_content = Paragraph::new(step_type).block(step_block);
        frame.render_widget(step_content, step_chunks[i]);

        // Draw a connecting arrow from the previous step
        if i > 0 {
            let prev_chunk = step_chunks[i - 1];
            let current_chunk = step_chunks[i];
            let arrow_line = Line::from("      ▼      ")
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(
                arrow_line,
                Rect {
                    x: prev_chunk.x,
                    y: prev_chunk.y + prev_chunk.height,
                    width: prev_chunk.width,
                    height: 1,
                },
            );
        }
    }
}
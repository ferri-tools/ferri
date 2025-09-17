//! TUI for real-time `ferri flow run` execution.

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ferri_core::flow::{Pipeline, StepUpdate, StepStatus};
use ratatui::{prelude::*, widgets::*};
use std::{io, time::Duration, collections::HashMap};
use crossbeam_channel::{Receiver, unbounded};
use std::thread;

struct AppState {
    steps: Vec<(String, StepStatus)>,
    outputs: HashMap<String, Vec<String>>,
    receiver: Receiver<StepUpdate>,
    active_step_name: String,
    is_done: bool,
}

pub fn run(pipeline: Pipeline) -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let (tx, rx) = unbounded();

    let initial_active_step = pipeline.steps.first().map_or(String::new(), |s| s.name.clone());

    let mut app_state = AppState {
        steps: pipeline.steps.iter().map(|s| (s.name.clone(), StepStatus::Pending)).collect(),
        outputs: HashMap::new(),
        receiver: rx,
        active_step_name: initial_active_step,
        is_done: false,
    };

    let base_path = std::env::current_dir()?;
    thread::spawn(move || {
        let _ = ferri_core::flow::run_pipeline(&base_path, &pipeline, tx);
    });

    while !app_state.is_done {
        terminal.draw(|f| ui(f, &app_state))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        if let Ok(update) = app_state.receiver.try_recv() {
            app_state.active_step_name = update.name.clone();
            if let Some(step) = app_state.steps.iter_mut().find(|(name, _)| name == &update.name) {
                step.1 = update.status.clone();
            }
            if let Some(output) = update.output {
                app_state.outputs.entry(update.name).or_default().push(output);
            }
        }
        
        if !app_state.steps.iter().any(|(_, s)| matches!(s, StepStatus::Pending | StepStatus::Running)) {
            app_state.is_done = true;
        }
    }

    restore_terminal(&mut terminal)
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn ui(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    f.render_widget(Block::default().title("Ferri Flow Execution").borders(Borders::TOP), chunks[0]);

    let step_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(chunks[1]);

    let step_list: Vec<ListItem> = state.steps.iter().map(|(name, status)| {
        let (status_text, color) = match status {
            StepStatus::Pending => ("Pending", Color::DarkGray),
            StepStatus::Running => ("Running", Color::Blue),
            StepStatus::Completed => ("Completed", Color::Green),
            StepStatus::Failed(_) => ("Failed", Color::Red),
        };
        let style = if name == &state.active_step_name {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };
        ListItem::new(Line::from(vec![
            Span::styled(format!("[{}]"), Style::default().fg(color)),
            Span::raw(format!(" {}", name)),
        ])).style(style)
    }).collect();

    f.render_widget(List::new(step_list).block(Block::default().title("Steps").borders(Borders::ALL)), step_chunks[0]);

    let (output_title, output_block, output_text) = 
        if let Some((_, status)) = state.steps.iter().find(|(name, _)| name == &state.active_step_name) {
            match status {
                StepStatus::Failed(err) => (
                    "Error",
                    Block::default().title("Error").borders(Borders::ALL).border_style(Style::default().fg(Color::Red)),
                    err.clone()
                ),
                _ => (
                    "Output",
                    Block::default().title("Output").borders(Borders::ALL),
                    state.outputs.get(&state.active_step_name).cloned().unwrap_or_default().join("\n")
                )
            }
        } else {
            ("Output", Block::default().title("Output").borders(Borders::ALL), "No active step.".to_string())
        };

    let output_widget = Paragraph::new(output_text)
        .block(output_block)
        .wrap(Wrap { trim: false });
    f.render_widget(output_widget, step_chunks[1]);
}
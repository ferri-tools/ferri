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
    active_step_index: usize,
    is_done: bool,
    fatal_error: Option<String>,
}

use atty::Stream;

pub fn run(pipeline: Pipeline) -> io::Result<()> {
    if !atty::is(Stream::Stdout) {
        return run_plain(pipeline);
    }

    let mut terminal = setup_terminal()?;
    let (tx, rx) = unbounded();

    let mut app_state = AppState {
        steps: pipeline.steps.iter().map(|s| (s.name.clone(), StepStatus::Pending)).collect(),
        outputs: HashMap::new(),
        receiver: rx,
        active_step_index: 0,
        is_done: false,
        fatal_error: None,
    };

    let base_path = std::env::current_dir()?;
    thread::spawn(move || {
        if let Err(e) = ferri_core::flow::run_pipeline(&base_path, &pipeline, tx.clone()) {
            tx.send(StepUpdate {
                name: "[FATAL]".to_string(),
                status: StepStatus::Failed(e.to_string()),
                output: None,
            }).unwrap();
        }
    });

    'main_loop: loop {
        terminal.draw(|f| ui(f, &app_state))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => {
                        app_state.active_step_index = app_state.active_step_index.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        app_state.active_step_index = (app_state.active_step_index + 1).min(app_state.steps.len() - 1);
                    }
                    _ => {}
                }
            }
        }

        if !app_state.is_done {
            if let Ok(update) = app_state.receiver.try_recv() {
                if update.name == "[FATAL]" {
                    if let StepStatus::Failed(err) = update.status {
                        app_state.fatal_error = Some(err.clone());
                        app_state.steps.push(("[FATAL]".to_string(), StepStatus::Failed(err)));
                        app_state.is_done = true;
                        // We need to draw one last time to show the error, then we can exit.
                        terminal.draw(|f| ui(f, &app_state))?;
                        break 'main_loop;
                    }
                } else {
                    if let Some((idx, step)) = app_state.steps.iter_mut().enumerate().find(|(_, (name, _))| name == &update.name) {
                        step.1 = update.status.clone();
                        app_state.active_step_index = idx;
                    }
                    if let Some(output) = update.output {
                        app_state.outputs.entry(update.name).or_default().push(output);
                    }
                }
            }
            
            if !app_state.is_done && !app_state.steps.iter().any(|(_, s)| matches!(s, StepStatus::Pending | StepStatus::Running)) {
                app_state.is_done = true;
            }
        }
    }

    restore_terminal(&mut terminal)?;

    if let Some(err) = app_state.fatal_error {
        return Err(io::Error::new(io::ErrorKind::Other, err));
    }

    Ok(())
}

fn run_plain(pipeline: Pipeline) -> io::Result<()> {
    println!("Non-interactive mode detected. Streaming output:");
    let (tx, rx) = unbounded();
    let base_path = std::env::current_dir()?;

    thread::spawn(move || {
        let _ = ferri_core::flow::run_pipeline(&base_path, &pipeline, tx);
    });

    for update in rx {
        match update.status {
            StepStatus::Running => {
                println!("[RUNNING] {}", update.name);
            }
            StepStatus::Completed => {
                println!("[COMPLETED] {}", update.name);
            }
            StepStatus::Failed(e) => {
                println!("[FAILED] {}: {}", update.name, e);
            }
            _ => {}
        }
        if let Some(output) = update.output {
            for line in output.lines() {
                println!("  | {}", line);
            }
        }
    }
    Ok(())
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
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(f.size());

    f.render_widget(Block::default().title("Ferri Flow Execution").borders(Borders::TOP), chunks[0]);
    f.render_widget(Paragraph::new("Use ↑/↓ to select a step. Press 'q' to quit.").alignment(Alignment::Center), chunks[2]);

    let step_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(chunks[1]);

    let step_list_items: Vec<ListItem> = state.steps.iter().enumerate().map(|(i, (name, status))| {
        let (status_text, color) = match status {
            StepStatus::Pending => ("Pending", Color::DarkGray),
            StepStatus::Running => ("Running", Color::Blue),
            StepStatus::Completed => ("Completed", Color::Green),
            StepStatus::Failed(_) => ("Failed", Color::Red),
        };
        let style = if i == state.active_step_index {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };
        ListItem::new(Line::from(vec![
            Span::styled(format!("[{}]", status_text), Style::default().fg(color)),
            Span::raw(format!(" {}", name)),
        ])).style(style)
    }).collect();

    f.render_widget(List::new(step_list_items).block(Block::default().title("Steps").borders(Borders::ALL)), step_chunks[0]);

    let (active_step_name, active_status) = &state.steps[state.active_step_index];
    let (_output_title, output_block, output_text) = match active_status {
        StepStatus::Failed(err) => (
            "Error",
            Block::default().title("Error").borders(Borders::ALL).border_style(Style::default().fg(Color::Red)),
            err.clone()
        ),
        _ => (
            "Output",
            Block::default().title(format!("Output for '{}'", active_step_name)).borders(Borders::ALL),
            state.outputs.get(active_step_name).cloned().unwrap_or_default().join("\n")
        )
    };

    let output_widget = Paragraph::new(output_text)
        .block(output_block)
        .wrap(Wrap { trim: false });
    f.render_widget(output_widget, step_chunks[1]);
}

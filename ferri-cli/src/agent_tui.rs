use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::io;
use std::time::{Duration, Instant};
use ferri_agent;
use ferri_automation;

pub enum AgentState {
    Initializing,
    Executing(String),
    Failed(String),
    Success(String),
}

struct App {
    state: AgentState,
    quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            state: AgentState::Initializing,
            quit: false,
        }
    }
}

pub fn run(prompt: &str) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate, prompt);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
    prompt: &str,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    let base_path = std::env::current_dir()?;
    let prompt_clone = prompt.to_string();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let job_id_result =
                ferri_agent::agent::generate_and_run_flow(&base_path, &prompt_clone, |msg| {
                    tx.send(msg.to_string()).unwrap();
                })
                .await;

            let job_id = match job_id_result {
                Ok(id) => {
                    tx.send(format!("AGENT JOB ID: {}", id)).unwrap();
                    id
                }
                Err(e) => {
                    tx.send(format!("AGENT FAILED: {}", e)).unwrap();
                    return;
                }
            };

            // Polling loop
            loop {
                std::thread::sleep(Duration::from_millis(500));
                match ferri_automation::jobs::list_jobs(&base_path) {
                    Ok(jobs) => {
                        if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
                            let output =
                                ferri_automation::jobs::get_job_output(&base_path, &job_id).unwrap_or_default();
                            tx.send(format!("OUTPUT:\n{}", output)).unwrap();

                            match job.status.as_str() {
                                "Completed" => {
                                    tx.send("AGENT SUCCESS".to_string()).unwrap();
                                    break;
                                }
                                "Failed" => {
                                    tx.send(format!("AGENT FAILED: Job {} failed.", job_id))
                                        .unwrap();
                                    break;
                                }
                                _ => {} // Still running
                            }
                        }
                    }
                    Err(e) => {
                        tx.send(format!("AGENT FAILED: Could not list jobs: {}", e))
                            .unwrap();
                        break;
                    }
                }
            }
        });
    });

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Ok(msg) = rx.try_recv() {
            if msg.starts_with("AGENT FAILED:") {
                app.state = AgentState::Failed(msg);
                app.quit = true;
            } else if msg.starts_with("AGENT SUCCESS") {
                 if let AgentState::Executing(current_output) = &app.state {
                    app.state = AgentState::Success(current_output.clone());
                } else {
                    app.state = AgentState::Success("Flow completed successfully.".to_string());
                }
            } else if msg.starts_with("OUTPUT:") {
                let output = msg.strip_prefix("OUTPUT:\n").unwrap_or(&msg).to_string();
                app.state = AgentState::Executing(output);
            } else {
                 // Initial messages from the agent before execution
                if let AgentState::Initializing = &app.state {
                    app.state = AgentState::Executing(msg);
                }
            }
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    app.quit = true;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Footer
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("Ferri Agent")
        .style(Style::default().fg(Color::Magenta).bold())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let (status_text, status_style) = match &app.state {
        AgentState::Initializing => ("Initializing...".to_string(), Style::default().fg(Color::Yellow)),
        AgentState::Executing(output) => (output.clone(), Style::default().fg(Color::Cyan)),
        AgentState::Failed(err) => (err.clone(), Style::default().fg(Color::Red)),
        AgentState::Success(output) => (format!("{}\n\nFlow completed successfully. Press 'q' to quit.", output), Style::default().fg(Color::Green)),
    };

    let content_block = Block::default().title("Agent Status").borders(Borders::ALL);
    let content = Paragraph::new(status_text)
        .style(status_style)
        .block(content_block)
        .wrap(Wrap { trim: true });
    f.render_widget(content, chunks[1]);

    let footer_text = match app.state {
        AgentState::Success(_) | AgentState::Failed(_) => "Press 'q' to quit.",
        _ => "Executing... Press 'q' to quit.",
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

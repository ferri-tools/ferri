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

pub enum AgentState {
    Initializing,
    ExecutingStep(Vec<String>),
    Failed(String),
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
    let (tx, rx) = std::sync::mpsc::channel();

    let base_path = std::env::current_dir()?;
    let prompt_clone = prompt.to_string();

    // Spawn a new Tokio runtime in a separate thread to run the async agent
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let result = ferri_core::agent::generate_and_run_flow(&base_path, &prompt_clone, |msg| {
                tx.send(msg.to_string()).unwrap();
            })
            .await;

            if let Err(e) = result {
                tx.send(format!("AGENT FAILED: {}", e)).unwrap();
            }
        });
    });

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // Check for new messages from the agent
        if let Ok(msg) = rx.try_recv() {
            if msg.starts_with("AGENT FAILED:") {
                app.state = AgentState::Failed(msg);
                app.quit = true; // Quit on failure to show the error
            } else {
                if let AgentState::ExecutingStep(ref mut steps) = &mut app.state {
                    steps.push(msg);
                } else {
                    app.state = AgentState::ExecutingStep(vec![msg]);
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
            // A small delay to let the user see the final message
            std::thread::sleep(Duration::from_secs(2));
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
        AgentState::ExecutingStep(steps) => {
            if steps.is_empty() {
                ("Initializing...".to_string(), Style::default().fg(Color::Yellow))
            } else {
                (steps.join("\n"), Style::default().fg(Color::Green))
            }
        }
        AgentState::Failed(err) => (err.clone(), Style::default().fg(Color::Red)),
    };

    let content_block = Block::default().title("Agent Status").borders(Borders::ALL);
    let content = Paragraph::new(status_text)
        .style(status_style)
        .block(content_block)
        .wrap(Wrap { trim: true });
    f.render_widget(content, chunks[1]);

    let footer = Paragraph::new("Press 'q' to quit.")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

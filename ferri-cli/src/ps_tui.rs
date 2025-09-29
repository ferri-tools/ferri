use chrono::Utc;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ferri_automation::jobs::Job;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};
use std::io;
use std::time::{Duration, Instant};

struct App {
    jobs: Vec<Job>,
    table_state: TableState,
}

impl App {
    fn new(jobs: Vec<Job>) -> Self {
        let mut table_state = TableState::default();
        if !jobs.is_empty() {
            table_state.select(Some(0));
        }
        Self { jobs, table_state }
    }

    pub fn next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.jobs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.jobs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
}

pub fn run(jobs: Vec<Job>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(1000); // Refresh every second
    let app = App::new(jobs);
    let res = run_app(&mut terminal, app, tick_rate);

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
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => app.next(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            // Here you would re-fetch the jobs from ferri_core
            // app.jobs = ferri_core::jobs::list_jobs(...)?;
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.size());

    // Header
    let header = Paragraph::new(" F E R R I // P S // D A S H B O A R D ")
        .style(
            Style::default()
                .fg(Color::Magenta)
                .bg(Color::Black)
                .bold(),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Main Content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    // Job Table
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let header_cells = ["ID", "Status", "Command", "Start Time", "Duration"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan).bold()));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Black))
        .height(1);

    let rows = app.jobs.iter().map(|job| {
        let status_style = match job.status.as_str() {
            "Running" => Style::default().fg(Color::Green),
            "Completed" => Style::default().fg(Color::Blue),
            "Failed" | "Terminated" => Style::default().fg(Color::Red),
            _ => Style::default(),
        };
        let duration = Utc::now() - job.start_time;
        let duration_str = format_duration(duration);
        let cells = vec![
            Cell::from(job.id.clone()),
            Cell::from(job.status.clone()).style(status_style),
            Cell::from(job.command.clone()),
            Cell::from(job.start_time.format("%Y-%m-%d %H:%M:%S").to_string()),
            Cell::from(duration_str),
        ];
        Row::new(cells).height(1)
    });

    let table = Table::new(
        rows,
        &[
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Min(30),
            Constraint::Length(20),
            Constraint::Length(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Active Jobs "),
    )
    .highlight_style(selected_style)
    .highlight_symbol(">> ");
    f.render_stateful_widget(table, main_chunks[0], &mut app.table_state);

    // Job Details View
    let details_block = Block::default().borders(Borders::ALL);
    let details_content = if let Some(selected_index) = app.table_state.selected() {
        if let Some(job) = app.jobs.get(selected_index) {
            if job.status == "Failed" {
                if let Some(error_preview) = &job.error_preview {
                    Paragraph::new(error_preview.as_str())
                        .block(details_block.title(" Error Preview (stderr) "))
                        .wrap(ratatui::widgets::Wrap { trim: true })
                } else {
                    Paragraph::new("No error details available.")
                        .block(details_block.title(" Details "))
                }
            } else {
                Paragraph::new(job.command.as_str())
                    .block(details_block.title(" Job Details "))
                    .wrap(ratatui::widgets::Wrap { trim: true })
            }
        } else {
            Paragraph::new("").block(details_block.title(" Details "))
        }
    } else {
        Paragraph::new("Select a job to see details.")
            .block(details_block.title(" Details "))
    };
    f.render_widget(details_content, main_chunks[1]);

    // Footer
    let footer = Paragraph::new("Use (j/k) or (↑/↓) to navigate. Press (q) to quit.")
        .style(Style::default().fg(Color::Yellow).bg(Color::Black))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

fn format_duration(duration: chrono::Duration) -> String {
    let secs = duration.num_seconds();
    if secs < 0 {
        return "0s".to_string();
    }
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

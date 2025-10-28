use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ferri_automation::flow::{JobStatus, Update};
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};

struct App {
    jobs: HashMap<String, JobStatus>,
    job_outputs: HashMap<String, Vec<String>>,
    job_order: Vec<String>,
    selected_job: usize,
    total_jobs: usize,
    quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            job_outputs: HashMap::new(),
            job_order: Vec::new(),
            selected_job: 0,
            total_jobs: 0,
            quit: false,
        }
    }

    fn on_update(&mut self, update: Update) {
        match update {
            Update::Job(job_update) => {
                if !self.jobs.contains_key(&job_update.job_id) {
                    self.total_jobs += 1;
                    self.job_order.push(job_update.job_id.clone());
                }
                self.jobs
                    .insert(job_update.job_id.clone(), job_update.status);
            }
            Update::Step(_) => {
                // For now, we only display job-level status
            }
            Update::Output(output_update) => {
                self.job_outputs
                    .entry(output_update.job_id)
                    .or_default()
                    .push(output_update.line);
            }
        }
    }

    fn next_job(&mut self) {
        if !self.job_order.is_empty() {
            self.selected_job = (self.selected_job + 1) % self.job_order.len();
        }
    }

    fn previous_job(&mut self) {
        if !self.job_order.is_empty() {
            if self.selected_job > 0 {
                self.selected_job -= 1;
            } else {
                self.selected_job = self.job_order.len() - 1;
            }
        }
    }
}

pub fn run(log_path: &Path) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let mut app = App::new();

    std::thread::sleep(Duration::from_millis(100));
    let file = File::open(log_path)?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        while reader.read_line(&mut line)? > 0 {
            if let Ok(update) = serde_json::from_str::<Update>(&line) {
                app.on_update(update);
            }
            line.clear();
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => app.quit = true,
                    KeyCode::Down => app.next_job(),
                    KeyCode::Up => app.previous_job(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.size());

    // Jobs List (Left Pane)
    let mut job_items = vec![];
    for job_id in &app.job_order {
        let status = app.jobs.get(job_id).cloned().unwrap_or(JobStatus::Pending);
        let (status_text, style) = match status {
            JobStatus::Pending => ("⏳ Pending".to_string(), Style::default().fg(Color::Yellow)),
            JobStatus::Running => ("⚙️ Running".to_string(), Style::default().fg(Color::Cyan)),
            JobStatus::Succeeded => ("✅ Succeeded".to_string(), Style::default().fg(Color::Green)),
            JobStatus::Failed(e) => (format!("❌ Failed: {}", e), Style::default().fg(Color::Red)),
        };
        job_items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{:<20}", job_id), Style::default().bold()),
            Span::styled(status_text, style),
        ])));
    }

    let jobs_list = List::new(job_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Jobs"),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        )
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_job));

    f.render_stateful_widget(jobs_list, chunks[0], &mut list_state);

    // Output View (Right Pane)
    let selected_job_id = app.job_order.get(app.selected_job);
    let output_text = if let Some(job_id) = selected_job_id {
        app.job_outputs
            .get(job_id)
            .map(|lines| lines.join("\n"))
            .unwrap_or_else(|| "No output for this job yet.".to_string())
    } else {
        "Select a job to view its output.".to_string()
    };

    let output_paragraph = Paragraph::new(output_text)
        .block(Block::default().borders(Borders::ALL).title("Output"))
        .wrap(Wrap { trim: true });

    f.render_widget(output_paragraph, chunks[1]);
}

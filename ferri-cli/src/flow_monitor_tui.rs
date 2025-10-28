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
    total_jobs: usize, // To know when to exit
    quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            total_jobs: 0, // Will be updated as we see jobs
            quit: false,
        }
    }

    fn on_update(&mut self, update: Update) {
        match update {
            Update::Job(job_update) => {
                if !self.jobs.contains_key(&job_update.job_id) {
                    self.total_jobs += 1;
                }
                self.jobs.insert(job_update.job_id, job_update.status);
            }
            Update::Step(_) => {
                // For now, we only display job-level status
            }
        }
    }

    fn is_finished(&self) -> bool {
        if self.total_jobs == 0 {
            return false;
        }
        let finished_jobs = self.jobs.values().filter(|s| s.is_terminal()).count();
        finished_jobs == self.total_jobs
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

    // Wait briefly for the log file to be created
    std::thread::sleep(Duration::from_millis(100));
    let file = File::open(log_path)?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Check for new updates from the log file
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
                if key.code == KeyCode::Char('q') {
                    app.quit = true;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.quit || app.is_finished() {
            // Add a small delay to ensure the final status is rendered
            std::thread::sleep(Duration::from_millis(500));
            terminal.draw(|f| ui(f, &app))?;
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

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)])
        .split(f.size());

    let mut items = vec![];
    for (job_id, status) in &app.jobs {
        let (status_text, style) = match status {
            JobStatus::Pending => ("⏳ Pending".to_string(), Style::default().fg(Color::Yellow)),
            JobStatus::Running => ("⚙️ Running".to_string(), Style::default().fg(Color::Cyan)),
            JobStatus::Succeeded => ("✅ Succeeded".to_string(), Style::default().fg(Color::Green)),
            JobStatus::Failed(e) => {
                (format!("❌ Failed: {}", e), Style::default().fg(Color::Red))
            }
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{:<20}", job_id), Style::default().bold()),
            Span::styled(status_text, style),
        ])));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Flow Execution"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_widget(list, chunks[0]);
}

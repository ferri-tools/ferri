use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ferri_automation::flow::{self, JobStatus, StepStatus, Update};
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Clone)]
struct StepState {
    name: String,
    status: StepStatus,
    output: Vec<String>,
}

#[derive(Clone)]
struct JobState {
    name: String,
    status: JobStatus,
    steps: Vec<StepState>,
}

struct App {
    jobs: HashMap<String, JobState>,
    job_order: Vec<String>,
    selected_item: (usize, Option<usize>), // (job_index, Option<step_index>)
    flow_content: Option<String>,
    quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            job_order: Vec::new(),
            selected_item: (0, None),
            flow_content: None,
            quit: false,
        }
    }

    fn populate_from_flow(&mut self, flow_content: &str) {
        if let Ok(flow_doc) = serde_yaml::from_str::<flow::FlowDocument>(flow_content) {
            let mut job_order: Vec<String> = flow_doc.spec.jobs.keys().cloned().collect();
            job_order.sort(); // Sort for consistent order

            // Prepend "generating-flow" if it exists, otherwise just use the new order
            if let Some(index) = self.job_order.iter().position(|r| r == "generating-flow") {
                 let mut new_order = vec![self.job_order[index].clone()];
                 new_order.extend(job_order);
                 self.job_order = new_order;
            } else {
                self.job_order = job_order;
            }


            for (job_id, job_spec) in flow_doc.spec.jobs {
                let steps = job_spec
                    .steps
                    .iter()
                    .enumerate()
                    .map(|(i, s)| StepState {
                        name: s.name.clone().unwrap_or_else(|| format!("Step {}", i + 1)),
                        status: StepStatus::Pending,
                        output: Vec::new(),
                    })
                    .collect();

                self.jobs.insert(
                    job_id,
                    JobState {
                        name: job_spec.name.unwrap_or_else(|| "Unnamed Job".to_string()),
                        status: JobStatus::Pending,
                        steps,
                    },
                );
            }
        }
    }

    fn on_update(&mut self, update: Update) {
        match update {
            Update::Job(job_update) => {
                if let Some(job) = self.jobs.get_mut(&job_update.job_id) {
                    job.status = job_update.status;
                } else {
                    // This can happen for the initial "generating-flow" job
                    if !self.job_order.contains(&job_update.job_id) {
                        self.job_order.push(job_update.job_id.clone());
                    }
                    self.jobs.insert(
                        job_update.job_id,
                        JobState {
                            name: "Flow Generation".to_string(),
                            status: job_update.status,
                            steps: Vec::new(),
                        },
                    );
                }
            }
            Update::Step(step_update) => {
                if let Some(job) = self.jobs.get_mut(&step_update.job_id) {
                    if let Some(step) = job.steps.get_mut(step_update.step_index) {
                        step.status = step_update.status;
                    }
                }
            }
            Update::Output(output_update) => {
                if let Some(job) = self.jobs.get_mut(&output_update.job_id) {
                    if let Some(step) = job.steps.get_mut(output_update.step_index) {
                        step.output.push(output_update.line);
                    }
                }
            }
            Update::FlowFile(flow_file) => {
                self.flow_content = Some(flow_file.content);
                if let Some(content) = self.flow_content.clone() {
                    self.populate_from_flow(&content);
                }
            }
        }
    }

    fn next(&mut self) {
        let (job_idx, step_idx_opt) = self.selected_item;
        if self.job_order.is_empty() {
            return;
        }

        match step_idx_opt {
            None => {
                // Job is selected, try selecting its first step if it has any
                let job_id = &self.job_order[job_idx];
                if !self.jobs.get(job_id).map_or(true, |j| j.steps.is_empty()) {
                    self.selected_item = (job_idx, Some(0));
                } else if job_idx + 1 < self.job_order.len() {
                    // Otherwise, select the next job
                    self.selected_item = (job_idx + 1, None);
                }
            }
            Some(step_idx) => {
                let job_id = &self.job_order[job_idx];
                let num_steps = self.jobs.get(job_id).map_or(0, |j| j.steps.len());
                if step_idx + 1 < num_steps {
                    // Select next step in the same job
                    self.selected_item = (job_idx, Some(step_idx + 1));
                } else if job_idx + 1 < self.job_order.len() {
                    // Select next job
                    self.selected_item = (job_idx + 1, None);
                }
            }
        }
    }

    fn previous(&mut self) {
        let (job_idx, step_idx_opt) = self.selected_item;
        if self.job_order.is_empty() {
            return;
        }

        match step_idx_opt {
            None => {
                // Job is selected, try selecting previous job
                if job_idx > 0 {
                    self.selected_item = (job_idx - 1, None);
                }
            }
            Some(0) => {
                // First step is selected, so select the parent job
                self.selected_item = (job_idx, None);
            }
            Some(step_idx) => {
                // Go to previous step in the same job
                self.selected_item = (job_idx, Some(step_idx - 1));
            }
        }
    }
}

pub fn run(log_path: &Path, _initial_flow_content: Option<String>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(100);
    let mut app = App::new();

    let (tx, rx) = mpsc::channel();
    let log_path_buf = log_path.to_path_buf();

    thread::spawn(move || {
        // Wait a moment for the log file to be created by the main thread
        thread::sleep(Duration::from_millis(100));

        // This loop will retry opening the file, in case the watcher starts before the file exists.
        let file = loop {
            match File::open(&log_path_buf) {
                Ok(f) => break f,
                Err(_) => {
                    // If it fails, wait a bit and try again.
                    thread::sleep(Duration::from_millis(50));
                }
            }
        };

        let mut reader = BufReader::new(file);
        let mut line = String::new();
        // This is a blocking read, which is fine for a background thread.
        // It will read until EOF, then wait for more lines to be added.
        while reader.read_line(&mut line).unwrap_or(0) > 0 {
            if let Ok(update) = serde_json::from_str::<Update>(&line) {
                if tx.send(update).is_err() {
                    // The receiver has disconnected, so we can exit.
                    break;
                }
            }
            line.clear();
        }
    });

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // Non-blocking check for updates from the watcher thread.
        // We loop on try_recv to drain the channel of all pending updates
        // on each tick of the TUI loop.
        while let Ok(update) = rx.try_recv() {
            app.on_update(update);
        }

        if crossterm::event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => app.quit = true,
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    _ => {}
                }
            }
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
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(f.size());

    // Left Pane: Tree view of jobs and steps
    let mut items = vec![];
    for (job_idx, job_id) in app.job_order.iter().enumerate() {
        let job_state = app.jobs.get(job_id).cloned().unwrap_or(JobState {
            name: "Unknown Job".to_string(),
            status: JobStatus::Pending,
            steps: vec![],
        });
        let (status_text, style) = match &job_state.status {
            JobStatus::Pending => ("⏳ Pending", Style::default().fg(Color::Yellow)),
            JobStatus::Running => ("⚙️ Running", Style::default().fg(Color::Cyan)),
            JobStatus::Succeeded => ("✅ Succeeded", Style::default().fg(Color::Green)),
            JobStatus::Failed(_) => ("❌ Failed", Style::default().fg(Color::Red)),
        };

        let is_selected = app.selected_item.0 == job_idx && app.selected_item.1.is_none();
        let item_style = if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };
        items.push(
            ListItem::new(Line::from(vec![
                Span::styled(format!("▶ {}", job_id), style.bold()),
                Span::raw(" "),
                Span::styled(status_text, style),
            ]))
            .style(item_style),
        );

        for (step_idx, step_state) in job_state.steps.iter().enumerate() {
            let (status_text, style) = match &step_state.status {
                StepStatus::Pending => ("⏳ Pending", Style::default().fg(Color::DarkGray)),
                StepStatus::Running => ("⚙️ Running", Style::default().fg(Color::Cyan)),
                StepStatus::Completed => ("✅ Completed", Style::default().fg(Color::Green)),
                StepStatus::Failed(_) => ("❌ Failed", Style::default().fg(Color::Red)),
            };
            let is_selected =
                app.selected_item.0 == job_idx && app.selected_item.1 == Some(step_idx);
            let item_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            items.push(
                ListItem::new(Line::from(vec![
                    Span::raw("  - "),
                    Span::styled(step_state.name.clone(), style.bold()),
                    Span::raw(" "),
                    Span::styled(status_text, style),
                ]))
                .style(item_style),
            );
        }
    }

    let tree = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Execution Plan"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_widget(tree, chunks[0]);

    // Right Pane: Output or Flow Content
    let (right_pane_title, output_text) = {
        let (job_idx, step_idx_opt) = app.selected_item;
        if let Some(job_id) = app.job_order.get(job_idx) {
            if let Some(step_idx) = step_idx_opt {
                // A step is selected, show its output
                let step_name = app.jobs.get(job_id)
                    .and_then(|j| j.steps.get(step_idx))
                    .map(|s| s.name.clone())
                    .unwrap_or_default();
                let title = format!("Output: {}", step_name);
                let content = app.jobs.get(job_id)
                    .and_then(|j| j.steps.get(step_idx))
                    .map(|s| s.output.join(""))
                    .unwrap_or_else(|| "No output for this step yet.".to_string());
                (title, content)
            } else {
                // A job is selected, show a summary or the flow content
                let job = app.jobs.get(job_id).unwrap();
                if let JobStatus::Failed(e) = &job.status {
                    ("Error".to_string(), format!("Job failed: {}", e))
                } else if let Some(content) = &app.flow_content {
                    ("Generated Flow".to_string(), content.clone())
                } else {
                    ("Output".to_string(), "Select a step to view its output.".to_string())
                }
            }
        } else {
            // Nothing is selected, show the flow content if available
            if let Some(content) = &app.flow_content {
                ("Generated Flow".to_string(), content.clone())
            } else {
                ("Status".to_string(), "Waiting for flow generation...".to_string())
            }
        }
    };

    let right_block = Block::default().borders(Borders::ALL).title(right_pane_title);
    let output_paragraph = Paragraph::new(output_text)
        .block(right_block)
        .wrap(Wrap { trim: false });

    f.render_widget(output_paragraph, chunks[1]);
}

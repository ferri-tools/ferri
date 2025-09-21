use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::canvas::{self, Canvas};
use std::{error::Error, io};

// App state and data
struct App {
    tasks: Vec<Task>,
}

struct Task {
    branch: String,
    branch_color: Color,
    task_name: String,
    icon: &'static str,
    status: String,
    graph_info: GraphInfo,
}

// Represents the visual elements in the graph column
struct GraphInfo {
    dot: bool,          // Does this row have a dot '●'?
    is_merge: bool,     // Is this a merge commit?
    is_fork: bool,      // Is this a new branch fork?
    connections: Vec<GraphConnection>,
}

enum GraphConnection {
    Vertical(usize), // A vertical line at a specific x-offset
    Fork(usize, usize), // A line forking from x1 to x2
    Merge(usize, usize), // A line merging from x1 to x2
}


impl App {
    fn new() -> App {
        // Define the data to be rendered, closely matching the screenshot
        let tasks = vec![
            Task {
                branch: "main".into(), branch_color: Color::Rgb(138, 173, 255), icon: "🔀", task_name: "Initial Commit".into(), status: "Queued".into(),
                graph_info: GraphInfo { dot: true, is_merge: false, is_fork: false, connections: vec![GraphConnection::Vertical(2)] }
            },
            Task {
                branch: "main".into(), branch_color: Color::Rgb(138, 173, 255), icon: "🛠️", task_name: "Setup Project".into(), status: "Queued".into(),
                graph_info: GraphInfo { dot: true, is_merge: false, is_fork: false, connections: vec![GraphConnection::Vertical(2)] }
            },
            Task {
                branch: "feature-branch".into(), branch_color: Color::Rgb(199, 160, 255), icon: "✨", task_name: "Start Feature".into(), status: "Queued".into(),
                graph_info: GraphInfo { dot: true, is_merge: false, is_fork: true, connections: vec![GraphConnection::Vertical(2), GraphConnection::Fork(2, 6)] }
            },
            Task {
                branch: "main".into(), branch_color: Color::Rgb(138, 173, 255), icon: "🏗️", task_name: "Refactor Core API".into(), status: "Queued".into(),
                graph_info: GraphInfo { dot: true, is_merge: false, is_fork: false, connections: vec![GraphConnection::Vertical(2), GraphConnection::Vertical(6)] }
            },
            Task {
                branch: "feature-branch".into(), branch_color: Color::Rgb(199, 160, 255), icon: "📊", task_name: "Analyze Requirements".into(), status: "Queued".into(),
                graph_info: GraphInfo { dot: true, is_merge: false, is_fork: false, connections: vec![GraphConnection::Vertical(2), GraphConnection::Vertical(6)] }
            },
            Task {
                branch: "feature-branch".into(), branch_color: Color::Rgb(199, 160, 255), icon: "🧠", task_name: "Implement Logic".into(), status: "Queued".into(),
                graph_info: GraphInfo { dot: true, is_merge: false, is_fork: false, connections: vec![GraphConnection::Vertical(2), GraphConnection::Vertical(6)] }
            },
            Task {
                branch: "feature-branch".into(), branch_color: Color::Rgb(199, 160, 255), icon: "📄", task_name: "Generate Docs".into(), status: "Queued".into(),
                graph_info: GraphInfo { dot: true, is_merge: false, is_fork: false, connections: vec![GraphConnection::Vertical(2), GraphConnection::Vertical(6)] }
            },
            Task {
                branch: "main".into(), branch_color: Color::Rgb(138, 173, 255), icon: "🔀", task_name: "Merge Feature".into(), status: "Queued".into(),
                graph_info: GraphInfo { dot: true, is_merge: true, is_fork: false, connections: vec![GraphConnection::Vertical(2), GraphConnection::Merge(6, 2)] }
            },
            Task {
                branch: "main".into(), branch_color: Color::Rgb(138, 173, 255), icon: "🧪", task_name: "Run Integration Tests".into(), status: "Queued".into(),
                graph_info: GraphInfo { dot: true, is_merge: false, is_fork: false, connections: vec![GraphConnection::Vertical(2)] }
            },
            Task {
                branch: "release-prep".into(), branch_color: Color::Rgb(141, 229, 141), icon: "📦", task_name: "Prepare Release".into(), status: "Queued".into(),
                graph_info: GraphInfo { dot: true, is_merge: false, is_fork: true, connections: vec![GraphConnection::Vertical(2), GraphConnection::Fork(2, 6)] }
            },
            Task {
                branch: "release-prep".into(), branch_color: Color::Rgb(141, 229, 141), icon: "🚀", task_name: "Deploy to Staging".into(), status: "Queued".into(),
                graph_info: GraphInfo { dot: true, is_merge: false, is_fork: false, connections: vec![GraphConnection::Vertical(2), GraphConnection::Vertical(6)] }
            },
            Task {
                branch: "main".into(), branch_color: Color::Rgb(138, 173, 255), icon: "🎉", task_name: "Finalize Release".into(), status: "Queued".into(),
                graph_info: GraphInfo { dot: true, is_merge: true, is_fork: false, connections: vec![GraphConnection::Vertical(2), GraphConnection::Merge(6, 2)] }
            },
        ];
        App { tasks }
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    // Set up the terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if let KeyCode::Char('q') = key.code {
                return Ok(());
            }
        }
    }
}

// This function renders the user interface
fn ui(f: &mut Frame, app: &App) {
    let size = f.size();
    
    // Main background
    f.render_widget(Block::default().bg(Color::Rgb(18, 25, 38)), size);

    // Define the main layout with three columns
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // For the title
            Constraint::Min(0),    // For the content
        ])
        .split(size);

    // Title
    let title = Paragraph::new("⭐ Gemini Flow   Agentic Workflow Orchestrator")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);
    f.render_widget(title, chunks[0]);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Column 1: Branch
            Constraint::Length(20), // Column 2: Graph
            Constraint::Min(40),    // Column 3: Task Details
        ])
        .split(chunks[1]);

    // Headers
    let headers = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(content_chunks[0]);
    f.render_widget(
        Paragraph::new("BRANCH").bold().fg(Color::Gray),
        headers[0],
    );
    let headers = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(content_chunks[1]);
    f.render_widget(
        Paragraph::new("GRAPH").bold().fg(Color::Gray),
        headers[0],
    );
    let headers = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(content_chunks[2]);
    f.render_widget(
        Paragraph::new("TASK DETAILS").bold().fg(Color::Gray),
        headers[0],
    );

    // --- Column 1: Branch Names ---
    let branch_items: Vec<ListItem> = app.tasks.iter().map(|task| {
        let line = Span::styled(
            format!(" {} ", &task.branch),
            Style::default().bg(task.branch_color).fg(Color::Black).bold(),
        );
        // Each item is two lines high to create spacing
        ListItem::new(vec![Line::from(line), Line::from("")])
    }).collect();
    let branch_list = List::new(branch_items);
    let list_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(content_chunks[0]);
    f.render_widget(branch_list, list_layout[1]);


    // --- Column 2: The Graph ---
    let graph_canvas = Canvas::default()
        .background_color(Color::Rgb(18, 25, 38))
        .x_bounds([0.0, 20.0])
        .y_bounds([0.0, (app.tasks.len() * 2) as f64])
        .paint(|ctx| {
            for (i, task) in app.tasks.iter().enumerate() {
                let y = (app.tasks.len() * 2) as f64 - (i * 2) as f64 - 1.5;

                for conn in &task.graph_info.connections {
                    match conn {
                        GraphConnection::Vertical(x) => {
                            ctx.draw(&canvas::Line {
                                x1: *x as f64, y1: y - 1.0, x2: *x as f64, y2: y + 1.0,
                                color: task.branch_color,
                            });
                        }
                        GraphConnection::Fork(x1, x2) => {
                             ctx.print(*x1 as f64 - 0.5, y, "╭".fg(task.branch_color));
                             ctx.draw(&canvas::Line {
                                x1: *x1 as f64, y1: y, x2: *x2 as f64, y2: y,
                                color: task.branch_color,
                            });
                        }
                        GraphConnection::Merge(x1, x2) => {
                             ctx.print(*x1 as f64 - 0.5, y, "╰".fg(task.branch_color));
                             ctx.draw(&canvas::Line {
                                x1: *x1 as f64, y1: y, x2: *x2 as f64, y2: y,
                                color: task.branch_color,
                            });
                        }
                    }
                }

                if task.graph_info.dot {
                     let dot_x = if task.graph_info.is_fork || task.graph_info.is_merge {
                        2.0
                     } else if task.branch == "feature-branch" || task.branch == "release-prep" {
                        6.0
                     } else {
                        2.0
                     };
                     ctx.print(dot_x - 0.5, y, "●".fg(task.branch_color));
                }
            }
        });
    let canvas_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(content_chunks[1]);
    f.render_widget(graph_canvas, canvas_layout[1]);

    // --- Column 3: Task Details ---
    let task_items: Vec<ListItem> = app.tasks.iter().map(|task| {
            let line = Line::from(vec![
                Span::styled(format!("{} ", task.icon), Style::default().fg(task.branch_color)),
                Span::styled(format!("{}  ", task.task_name), Style::default().fg(Color::White)),
                Span::styled(task.status.to_string(), Style::default().fg(Color::DarkGray)),
            ]);
            // Each item is two lines high to create spacing
            ListItem::new(vec![line, Line::from("")])
        })
        .collect();

    let tasks_list = List::new(task_items);
    let details_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(content_chunks[2]);
    f.render_widget(tasks_list, details_layout[1]);
}


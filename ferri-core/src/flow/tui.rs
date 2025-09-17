//! TUI logic for `ferri flow show` inspired by the treetrunk example.

use crate::flow::{Pipeline, Step, StepKind};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::canvas::{self, Canvas};
use std::io::{self, Stdout};
use std::collections::HashMap;

type CrosstermTerminal = Terminal<CrosstermBackend<Stdout>>;

// Struct to hold all the application's state and data
struct App<'a> {
    steps: Vec<RenderableStep<'a>>,
}

// A version of a Step that has pre-calculated graph information
struct RenderableStep<'a> {
    step: &'a Step,
    lane: usize,
    color: Color,
    graph_info: GraphInfo,
}

// Represents the visual elements in the graph column for a single step
struct GraphInfo {
    dot: bool,
    connections: Vec<GraphConnection>,
}

enum GraphConnection {
    Vertical,
    TopLeftCorner,
    BottomLeftCorner,
    Horizontal,
}

// Main entry point for the TUI
pub fn run_tui(pipeline: &Pipeline) -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let app = App::from_pipeline(pipeline);
    let res = run_app(&mut terminal, app);
    restore_terminal(&mut terminal)?;
    if let Err(e) = res {
        eprintln!("TUI Error: {}", e);
    }
    Ok(())
}

impl<'a> App<'a> {
    // Pre-process the pipeline to calculate layout and graph information
    fn from_pipeline(pipeline: &'a Pipeline) -> Self {
        let mut lanes = HashMap::new();
        let mut next_lane = 0;
        let mut steps = Vec::new();
        let step_indices: HashMap<_, _> = pipeline.steps.iter().enumerate().map(|(i, s)| (s.name.as_str(), i)).collect();

        let colors = [
            Color::Rgb(138, 173, 255), Color::Rgb(199, 160, 255),
            Color::Rgb(141, 229, 141), Color::Cyan, Color::Magenta, Color::Yellow,
        ];

        for (i, step) in pipeline.steps.iter().enumerate() {
            let mut connections = vec![GraphConnection::Vertical];
            let lane = *lanes.entry(i).or_insert_with(|| {
                let l = next_lane;
                next_lane += 1;
                l
            });

            if let Some(input_name) = &step.input {
                 if let Some(&prev_index) = step_indices.get(input_name.as_str()) {
                    let prev_lane = *lanes.entry(prev_index).or_insert(lane);
                    if prev_lane != lane {
                        connections.push(GraphConnection::BottomLeftCorner);
                        for _ in 0..(lane as usize).abs_diff(prev_lane as usize) {
                            connections.push(GraphConnection::Horizontal);
                        }
                    }
                 }
            }

            let fork_count = pipeline.steps.iter().filter(|s| s.input.as_deref() == Some(&step.name)).count();
            if fork_count > 1 {
                connections.push(GraphConnection::TopLeftCorner);
                connections.push(GraphConnection::Horizontal);
            }

            steps.push(RenderableStep {
                step,
                lane,
                color: colors[lane % colors.len()],
                graph_info: GraphInfo { dot: true, connections },
            });
        }

        App { steps }
    }
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
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

// The main application loop
fn run_app(terminal: &mut CrosstermTerminal, app: App) -> io::Result<()> {
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
    f.render_widget(Block::default().bg(Color::Rgb(18, 25, 38)), size);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25), // Column 1: Step Name
            Constraint::Length(20), // Column 2: Graph
            Constraint::Min(40),    // Column 3: Details
        ])
        .split(size);

    // --- Column 1: Step Names ---
    let step_name_items: Vec<ListItem> = app.steps.iter().map(|rs| {
        let line = Span::styled(format!(" {} ", &rs.step.name), Style::default().bg(rs.color).fg(Color::Black).bold());
        ListItem::new(vec![Line::from(""), Line::from(line), Line::from("")])
    }).collect();
    f.render_widget(List::new(step_name_items), chunks[0]);

    // --- Column 2: The Graph ---
    let graph_canvas = Canvas::default()
        .background_color(Color::Rgb(18, 25, 38))
        .x_bounds([0.0, 20.0])
        .y_bounds([0.0, (app.steps.len() * 3) as f64])
        .paint(|ctx| {
            for (i, rs) in app.steps.iter().enumerate() {
                let y = (app.steps.len() * 3) as f64 - (i * 3) as f64 - 2.0;
                let x_base = (rs.lane * 4) as f64 + 2.0;

                for conn in &rs.graph_info.connections {
                    match conn {
                        GraphConnection::Vertical => ctx.draw(&canvas::Line { x1: x_base, y1: y - 1.5, x2: x_base, y2: y + 1.5, color: rs.color }),
                        GraphConnection::TopLeftCorner => ctx.print(x_base, y + 0.5, "╭".fg(rs.color)),
                        GraphConnection::BottomLeftCorner => ctx.print(x_base, y - 0.5, "╰".fg(rs.color)),
                        GraphConnection::Horizontal => ctx.draw(&canvas::Line { x1: x_base, y1: y + 0.5, x2: x_base + 4.0, y2: y + 0.5, color: rs.color }),
                    }
                }
                if rs.graph_info.dot {
                    ctx.print(x_base - 0.5, y, "●".fg(rs.color));
                }
            }
        });
    f.render_widget(graph_canvas, chunks[1]);

    // --- Column 3: Task Details ---
    let detail_items: Vec<ListItem> = app.steps.iter().map(|rs| {
        let (kind_str, icon) = match rs.step.kind {
            StepKind::Model(_) => ("Model", "🧠"),
            StepKind::Process(_) => ("Process", "🛠️"),
        };
        let input_str = format!("Input: {}", rs.step.input.as_deref().unwrap_or("stdin"));
        let output_str = format!("Output: {}", rs.step.output.as_deref().unwrap_or("stdout"));
        let line1 = Line::from(vec![Span::styled(format!("{} ", icon), Style::default().fg(rs.color)), Span::styled(kind_str, Style::default().fg(Color::White))]);
        let line2 = Line::from(vec![Span::raw("  "), Span::styled(input_str, Style::default().fg(Color::DarkGray)), Span::raw(" -> "), Span::styled(output_str, Style::default().fg(Color::DarkGray))]);
        ListItem::new(vec![line1, line2, Line::from("")])
    }).collect();
    f.render_widget(List::new(detail_items), chunks[2]);
}
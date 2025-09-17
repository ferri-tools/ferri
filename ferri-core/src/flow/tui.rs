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
        let mut steps = Vec::new();
        let step_indices: HashMap<_, _> = pipeline.steps.iter().enumerate().map(|(i, s)| (s.name.as_str(), i)).collect();
        let children_map: HashMap<usize, Vec<usize>> = pipeline.steps.iter().enumerate().fold(HashMap::new(), |mut acc, (i, _)| {
            if let Some(input_name) = &pipeline.steps[i].input {
                if let Some(&parent_index) = step_indices.get(input_name.as_str()) {
                    acc.entry(parent_index).or_default().push(i);
                }
            }
            acc
        });

        let colors = [
            Color::Rgb(138, 173, 255), Color::Rgb(199, 160, 255),
            Color::Rgb(141, 229, 141), Color::Cyan, Color::Magenta, Color::Yellow,
        ];

        let mut lanes: HashMap<usize, usize> = HashMap::new();
        let mut active_lanes: Vec<Option<usize>> = vec![];

        for i in 0..pipeline.steps.len() {
            let parent_index = step_indices.get(&pipeline.steps[i].input.clone().unwrap_or_default() as &str);

            let lane = if let Some(&p_idx) = parent_index {
                let parent_lane = lanes.get(&p_idx).copied().unwrap_or(0);
                let siblings = children_map.get(&p_idx).unwrap();
                let fork_index = siblings.iter().position(|&s_idx| s_idx == i).unwrap_or(0);

                if fork_index == 0 {
                    parent_lane
                } else {
                    let mut new_lane = parent_lane + fork_index; // Use fork_index for spacing
                    while active_lanes.iter().any(|&l| l == Some(new_lane)) {
                        new_lane += 1;
                    }
                    new_lane
                }
            } else {
                0
            };

            if active_lanes.len() <= lane {
                active_lanes.resize(lane + 1, None);
            }
            active_lanes[lane] = Some(i);
            lanes.insert(i, lane);

            let parent_color = parent_index.and_then(|p| steps.iter().find(|s| s.step.name == pipeline.steps[*p].name)).map(|s| s.color).unwrap_or(colors[lane % colors.len()]);

            steps.push(RenderableStep {
                step: &pipeline.steps[i],
                lane,
                color: parent_color,
                graph_info: GraphInfo { dot: true },
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
            // First, draw all the vertical lane lines to connect the graph
            let step_positions: HashMap<_,_> = app.steps.iter().enumerate().map(|(i, rs)| (&rs.step.name, i)).collect();
            for lane_idx in 0..app.steps.iter().map(|s| s.lane).max().unwrap_or(0) + 1 {
                let lane_steps: Vec<_> = app.steps.iter().filter(|s| s.lane == lane_idx).collect();
                if !lane_steps.is_empty() {
                    let first_step_idx = step_positions.get(&lane_steps.first().unwrap().step.name).unwrap();
                    let last_step_idx = step_positions.get(&lane_steps.last().unwrap().step.name).unwrap();

                    let y_start = (app.steps.len() * 3) as f64 - (last_step_idx * 3) as f64 - 2.0;
                    let y_end = (app.steps.len() * 3) as f64 - (first_step_idx * 3) as f64 - 2.0;
                    let x = (lane_idx * 4) as f64 + 2.0;
                    ctx.draw(&canvas::Line { x1: x, y1: y_start, x2: x, y2: y_end, color: lane_steps.first().unwrap().color });
                }
            }

            for (i, rs) in app.steps.iter().enumerate() {
                let y = (app.steps.len() * 3) as f64 - (i * 3) as f64 - 2.0;
                let x_base = (rs.lane * 4) as f64 + 2.0;

                // Draw connections from parent
                if let Some(input_name) = &rs.step.input {
                    if let Some(prev_rs) = app.steps.iter().find(|s| s.step.name == *input_name) {
                        let prev_x_base = (prev_rs.lane * 4) as f64 + 2.0;
                        if prev_rs.lane != rs.lane { // It's a merge
                            ctx.print(x_base, y - 0.5, "╰".fg(rs.color));
                            ctx.draw(&canvas::Line { x1: prev_x_base, y1: y - 0.5, x2: x_base, y2: y - 0.5, color: prev_rs.color });
                            // Add label for merge
                            let label = input_name.to_string();
                            ctx.print(prev_x_base + 1.0, y - 1.5, label.fg(Color::DarkGray));
                        }
                    }
                }

                // Draw connections to children (forks)
                let children: Vec<_> = app.steps.iter().filter(|s| s.step.input.as_deref() == Some(&rs.step.name)).collect();
                if children.len() > 1 {
                    for (fork_idx, child_rs) in children.iter().enumerate() {
                        if fork_idx > 0 {
                            let child_x_base = (child_rs.lane * 4) as f64 + 2.0;
                            ctx.print(x_base, y + 0.5, "╭".fg(rs.color));
                            ctx.draw(&canvas::Line { x1: x_base, y1: y + 0.5, x2: child_x_base, y2: y + 0.5, color: rs.color });
                        }
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
//! TUI logic for `ferri flow show` using a classic node-vertex graph representation.

use crate::flow::{Pipeline, Step};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::canvas::{self, Canvas};
use std::io::{self, Stdout};
use std::collections::{HashMap, VecDeque};

type CrosstermTerminal = Terminal<CrosstermBackend<Stdout>>;

// Represents a node in our graph with calculated coordinates
struct Node<'a> {
    step: &'a Step,
    id: usize,
    x: f64,
    y: f64,
}

// Main application state
struct App<'a> {
    nodes: Vec<Node<'a>>,
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
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
    // Calculate node positions using a topological sort
    fn from_pipeline(pipeline: &'a Pipeline) -> Self {
        let mut nodes = Vec::new();
        if pipeline.steps.is_empty() {
            return App { nodes, x_bounds: [0.0, 100.0], y_bounds: [0.0, 100.0] };
        }

        let step_indices: HashMap<_, _> = pipeline.steps.iter().enumerate().map(|(i, s)| (s.name.as_str(), i)).collect();
        let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut in_degree = vec![0; pipeline.steps.len()];

        for (i, step) in pipeline.steps.iter().enumerate() {
            if let Some(input) = &step.input {
                if let Some(&prev_idx) = step_indices.get(input.as_str()) {
                    adj.entry(prev_idx).or_default().push(i);
                    in_degree[i] += 1;
                }
            }
        }

        let mut queue: VecDeque<usize> = (0..pipeline.steps.len()).filter(|&i| in_degree[i] == 0).collect();
        let mut levels: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut max_depth = 0;

        while !queue.is_empty() {
            levels.insert(max_depth, queue.drain(..).collect());
            let current_level = levels.get(&max_depth).unwrap();

            for &u in current_level {
                if let Some(neighbors) = adj.get(&u) {
                    for &v in neighbors {
                        in_degree[v] -= 1;
                        if in_degree[v] == 0 {
                            queue.push_back(v);
                        }
                    }
                }
            }
            max_depth += 1;
        }

        let x_step = 100.0 / (max_depth as f64 + 1.0);
        for (depth, steps_in_level) in &levels {
            let y_step = 100.0 / (steps_in_level.len() as f64 + 1.0);
            for (i, &step_idx) in steps_in_level.iter().enumerate() {
                nodes.push(Node {
                    step: &pipeline.steps[step_idx],
                    id: step_idx + 1,
                    x: x_step * (*depth as f64 + 1.0),
                    y: y_step * (i as f64 + 1.0),
                });
            }
        }
        
        nodes.sort_by_key(|n| n.id);

        App { nodes, x_bounds: [0.0, 100.0], y_bounds: [0.0, 100.0] }
    }
}

// Setup/restore terminal functions (unchanged)
fn setup_terminal() -> io::Result<CrosstermTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut CrosstermTerminal) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

// Main application loop (unchanged)
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

// This function renders the new UI
fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // For the graph
            Constraint::Length(app.nodes.len() as u16 + 2), // For the legend
        ])
        .split(f.size());

    // --- Graph Canvas ---
    let canvas = Canvas::default()
        .background_color(Color::Rgb(18, 25, 38))
        .x_bounds(app.x_bounds)
        .y_bounds(app.y_bounds)
        .paint(|ctx| {
            let node_map: HashMap<_, _> = app.nodes.iter().map(|n| (n.step.name.as_str(), n)).collect();

            for node in &app.nodes {
                if let Some(input_name) = &node.step.input {
                    if let Some(parent_node) = node_map.get(input_name.as_str()) {
                        ctx.draw(&canvas::Line {
                            x1: parent_node.x, y1: parent_node.y,
                            x2: node.x, y2: node.y,
                            color: Color::White,
                        });
                    }
                }
            }
            for node in &app.nodes {
                ctx.print(node.x - 1.0, node.y, format!("({})", node.id).fg(Color::Cyan));
            }
        });
    f.render_widget(canvas, chunks[0]);

    // --- Legend ---
    let legend_items: Vec<ListItem> = app.nodes.iter().map(|node| {
        ListItem::new(format!("{}: {}", node.id, node.step.name))
    }).collect();
    let legend = List::new(legend_items)
        .block(Block::default().title("Legend").borders(Borders::ALL));
    f.render_widget(legend, chunks[1]);
}
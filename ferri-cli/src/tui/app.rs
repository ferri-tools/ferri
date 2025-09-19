use crate::tui::widgets::process_widget::ProcessWidget;
use ferri_core::jobs::Job;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, TableState},
    Frame,
};

/// Represents the main application state.
pub struct App<'a> {
    pub jobs: &'a [Job],
    pub process_table_state: TableState,
}

impl<'a> App<'a> {
    /// Creates a new instance of the application.
    pub fn new(jobs: &'a [Job]) -> Self {
        Self {
            jobs,
            process_table_state: TableState::default(),
        }
    }

    /// This is the main drawing function for the TUI.
    /// It gets called on every frame/tick.
    pub fn draw(&mut self, f: &mut Frame) {
        // Create a main layout with 3 vertical chunks.
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Top bar
                Constraint::Min(0),    // Main content area
                Constraint::Length(3), // Footer
            ])
            .split(f.size());

        // --- Top Bar ---
        let top_bar = Paragraph::new("Ferri TUI Dashboard")
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Header"));
        f.render_widget(top_bar, main_chunks[0]);

        // --- Main Content ---
        // Split the main content area into two horizontal chunks.
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70), // Left side for processes/jobs
                Constraint::Percentage(30), // Right side for system info
            ])
            .split(main_chunks[1]);

        let process_widget = ProcessWidget::new(self.jobs);
        f.render_stateful_widget(process_widget, content_chunks[0], &mut self.process_table_state);

        // Split the right side for system info into vertical chunks.
        let system_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(content_chunks[1]);

        let cpu_widget = Block::default().title("CPU").borders(Borders::ALL);
        f.render_widget(cpu_widget, system_chunks[0]);

        let mem_widget = Block::default().title("Memory").borders(Borders::ALL);
        f.render_widget(mem_widget, system_chunks[1]);

        let net_widget = Block::default().title("Network").borders(Borders::ALL);
        f.render_widget(net_widget, system_chunks[2]);


        // --- Footer ---
        let footer = Paragraph::new("Press 'q' to quit.")
            .style(Style::default().fg(Color::LightCyan))
            .block(Block::default().borders(Borders::ALL).title("Controls"));
        f.render_widget(footer, main_chunks[2]);
    }
}

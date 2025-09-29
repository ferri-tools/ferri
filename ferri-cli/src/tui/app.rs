use crate::tui::widgets::{
    cpu_widget::CpuWidget, memory_widget::MemoryWidget, network_widget::NetworkWidget,
    process_widget::ProcessWidget,
};
use ferri_automation::jobs::Job;
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
    pub cpu_usage_data: Vec<u64>,
    pub mem_used: u64,
    pub mem_total: u64,
    pub net_up: u64,
    pub net_down: u64,
}

impl<'a> App<'a> {
    /// Creates a new instance of the application.
    pub fn new(jobs: &'a [Job]) -> Self {
        Self {
            jobs,
            process_table_state: TableState::default(),
            // Placeholder data
            cpu_usage_data: vec![
                10, 20, 15, 30, 25, 40, 50, 45, 60, 55, 70, 65, 80, 75, 90, 85, 100,
            ],
            mem_used: 6,
            mem_total: 16,
            net_up: 1234,
            net_down: 5678,
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
        f.render_stateful_widget(
            process_widget,
            content_chunks[0],
            &mut self.process_table_state,
        );

        // Split the right side for system info into vertical chunks.
        let system_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(content_chunks[1]);

        let cpu_widget = CpuWidget::new(&self.cpu_usage_data);
        cpu_widget.render(system_chunks[0], f.buffer_mut());

        let mem_widget = MemoryWidget::new(self.mem_used, self.mem_total);
        mem_widget.render(system_chunks[1], f.buffer_mut());

        let net_widget = NetworkWidget::new(self.net_up, self.net_down);
        net_widget.render(system_chunks[2], f.buffer_mut());


        // --- Footer ---
        let footer = Paragraph::new("Press 'q' to quit, 'j'/'k' to navigate.")
            .style(Style::default().fg(Color::LightCyan))
            .block(Block::default().borders(Borders::ALL).title("Controls"));
        f.render_widget(footer, main_chunks[2]);
    }

    pub fn next(&mut self) {
        let i = match self.process_table_state.selected() {
            Some(i) => {
                if i >= self.jobs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.process_table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.process_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.jobs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.process_table_state.select(Some(i));
    }
}

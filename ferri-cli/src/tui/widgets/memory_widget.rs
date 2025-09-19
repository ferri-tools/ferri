use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Widget},
};

/// A widget to display memory usage.
pub struct MemoryWidget {
    pub used: u64,
    pub total: u64,
}

impl MemoryWidget {
    pub fn new(used: u64, total: u64) -> Self {
        Self { used, total }
    }

    pub fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let ratio = if self.total > 0 {
            self.used as f64 / self.total as f64
        } else {
            0.0
        };

        let gauge = Gauge::default()
            .block(Block::default().title("Memory Usage").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(ratio)
            .label(format!("{:.2}%", ratio * 100.0));

        gauge.render(area, buf);
    }
}

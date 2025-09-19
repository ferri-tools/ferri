use ratatui::{
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Sparkline, Widget},
};

/// A widget to display CPU usage.
pub struct CpuWidget<'a> {
    pub data: &'a [u64],
}

impl<'a> CpuWidget<'a> {
    pub fn new(data: &'a [u64]) -> Self {
        Self { data }
    }

    pub fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title("CPU Usage (%)")
                    .borders(Borders::ALL),
            )
            .data(self.data)
            .style(Style::default().fg(Color::Green))
            .bar_set(symbols::bar::NINE_LEVELS);

        sparkline.render(area, buf);
    }
}

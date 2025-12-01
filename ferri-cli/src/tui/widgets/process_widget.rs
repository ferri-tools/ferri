use ferri_automation::jobs::JobInstance;
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, TableState},
};

/// A stateful widget to display a list of Ferri jobs.
pub struct ProcessWidget<'a> {
    jobs: &'a [JobInstance],
}

impl<'a> ProcessWidget<'a> {
    pub fn new(jobs: &'a [JobInstance]) -> Self {
        Self { jobs }
    }
}

impl StatefulWidget for ProcessWidget<'_> {
    type State = TableState;

    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer, state: &mut Self::State) {
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default().bg(Color::Blue);

        let header_cells = ["Job ID", "Status", "Command", "Started"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);

        let rows = self.jobs.iter().map(|job| {
            let cells = vec![
                Cell::from(job.id.clone()),
                Cell::from(format!("{:?}", job.status)),
                Cell::from(job.command.clone()),
                Cell::from(job.start_time.to_string()),
            ];
            Row::new(cells).height(1).bottom_margin(1)
        });

        let table = Table::new(rows, &[ratatui::layout::Constraint::Percentage(100)])
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Processes"))
            .highlight_style(selected_style)
            .widths([
                ratatui::layout::Constraint::Percentage(20),
                ratatui::layout::Constraint::Percentage(10),
                ratatui::layout::Constraint::Percentage(50),
                ratatui::layout::Constraint::Percentage(20),
            ]);

        StatefulWidget::render(table, area, buf, state);
    }
}

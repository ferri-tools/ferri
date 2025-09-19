use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};
/// A widget to display network I/O. 
pub struct NetworkWidget { 
    pub up: u64, 
    pub down: u64, 
}

impl NetworkWidget { 
    pub fn new(up: u64, down: u64) -> Self { 
        Self { up, down } 
    } 

    pub fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) { 
        let text = format!("Up: {} KB/s\nDown: {} KB/s", self.up, self.down); 
        let paragraph = Paragraph::new(text) 
            .style(Style::default().fg(Color::Yellow)) 
            .block(Block::default().title("Network I/O").borders(Borders::ALL)); 

        paragraph.render(area, buf); 
    } 
}


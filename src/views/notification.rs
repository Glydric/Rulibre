use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub(crate) struct NotificationState {
    pub(crate) message: Option<Result<String, String>>,
    pub(crate) ticks: u8,
}

impl Default for NotificationState {
    fn default() -> Self {
        Self {
            message: None,
            ticks: 0,
        }
    }
}

impl NotificationState {
    pub fn set(&mut self, result: Result<String, String>) {
        self.message = Some(result);
        self.ticks = 0;
    }

    /// Advance the auto-clear counter. Clears after ~3s at 250ms poll.
    pub fn tick(&mut self) {
        if self.message.is_some() {
            self.ticks += 1;
            if self.ticks >= 12 {
                self.message = None;
                self.ticks = 0;
            }
        }
    }
}

pub fn draw(app: &crate::app::App, frame: &mut Frame) {
    let Some(ref result) = app.notification.message else {
        return;
    };

    let (msg, color) = match result {
        Ok(msg) => (msg.as_str(), Color::Green),
        Err(msg) => (msg.as_str(), Color::Red),
    };

    let area = frame.area();
    let width = (msg.len() as u16 + 4).min(area.width.saturating_sub(2));
    let height = 3u16;
    let x = area.x + 1;
    let y = area.y + area.height.saturating_sub(height + 2); // above the status bar

    let box_area = Rect::new(x, y, width, height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(color));

    frame.render_widget(Clear, box_area);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            msg,
            Style::new().fg(color).bold(),
        )))
        .block(block),
        box_area,
    );
}

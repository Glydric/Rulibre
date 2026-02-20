use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use rulibre::config;
use rulibre::scanner;

use crate::app::{App, Mode};

#[derive(Default)]
pub(crate) struct SetupState {
    pub(crate) input: String,
    pub(crate) error: String,
}

pub fn draw(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Center a box: 60 wide, 7 tall
    let box_width = 60u16.min(area.width.saturating_sub(4));
    let box_height = 7u16;
    let x = area.x + (area.width.saturating_sub(box_width)) / 2;
    let y = area.y + (area.height.saturating_sub(box_height)) / 2;
    let box_area = Rect::new(x, y, box_width, box_height);

    let block = Block::default()
        .title(" Enter Calibre library path ")
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::LightBlue));

    let inner = block.inner(box_area);
    frame.render_widget(block, box_area);

    // Input line with cursor
    let input_line = Line::from(vec![
        Span::raw(&app.setup.input),
        Span::styled("█", Style::new().fg(Color::Yellow)),
    ]);
    frame.render_widget(Paragraph::new(input_line), inner);

    // Error message below input
    if !app.setup.error.is_empty() {
        let err_area = Rect::new(inner.x, inner.y + 2, inner.width, 1);
        frame.render_widget(
            Paragraph::new(Span::styled(&app.setup.error, Style::new().fg(Color::Red))),
            err_area,
        );
    }

    // Hint at bottom of box
    let hint_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(1),
        inner.width,
        1,
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("enter", Style::new().fg(Color::Yellow).bold()),
            Span::raw(" confirm  "),
            Span::styled("esc", Style::new().fg(Color::Yellow).bold()),
            Span::raw(" quit"),
        ])),
        hint_area,
    );
}

/// Returns `true` if the app should quit.
pub fn handle_key(app: &mut App, code: KeyCode) -> bool {
    match code {
        KeyCode::Esc => return true,
        KeyCode::Backspace => {
            app.setup.input.pop();
            app.setup.error.clear();
        }
        KeyCode::Char(c) => {
            app.setup.input.push(c);
            app.setup.error.clear();
        }
        KeyCode::Enter => {
            let path = config::sanitize_path(&app.setup.input);
            if path.is_empty() {
                app.setup.error = "No path provided.".to_string();
            } else {
                let p = std::path::Path::new(&path);
                if !p.is_dir() {
                    app.setup.error = format!("Path does not exist: {path}");
                } else if !config::is_calibre_library(p) {
                    app.setup.error =
                        "Not a valid Calibre library (missing metadata.db).".to_string();
                } else {
                    let cfg = config::Config {
                        library_path: path.clone(),
                    };
                    cfg.save();
                    let books = scanner::scan_library(p);
                    let len = books.len();
                    app.filtered_books = books.clone();
                    app.all_books = books;
                    app.scrollbar_state =
                        ratatui::widgets::ScrollbarState::new(len.saturating_sub(1));
                    app.table_state = ratatui::widgets::TableState::default();
                    if len > 0 {
                        app.table_state.select(Some(0));
                    }
                    app.mode = Mode::Normal;
                }
            }
        }
        _ => {}
    }
    false
}

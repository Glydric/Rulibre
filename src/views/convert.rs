use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use rulibre::converter;
use rulibre::scanner;

use crate::app::{App, Mode};

pub fn draw(app: &App, frame: &mut Frame) {
    let area = frame.area();

    let book_title = app
        .table_state
        .selected()
        .and_then(|i| app.filtered_books.get(i))
        .map(|b| b.title.as_str())
        .unwrap_or("Unknown");

    let title = format!(" Convert: {book_title} ");

    // Box height: targets list + message line + hint line + borders + padding
    let list_len = app.convert_targets.len().max(1);
    let box_height = (list_len as u16 + 4).min(area.height.saturating_sub(2));
    let box_width = 50u16.min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(box_width)) / 2;
    let y = area.y + (area.height.saturating_sub(box_height)) / 2;
    let box_area = Rect::new(x, y, box_width, box_height);

    frame.render_widget(Clear, box_area);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::LightBlue));

    let inner = block.inner(box_area);
    frame.render_widget(block, box_area);

    if app.convert_targets.is_empty() {
        // Only a message to display (error state)
        let msg_style = if app.convert_is_error {
            Style::new().fg(Color::Red)
        } else {
            Style::new().fg(Color::Green)
        };
        frame.render_widget(
            Paragraph::new(Span::styled(&app.convert_message, msg_style)),
            inner,
        );
        let hint_area = Rect::new(
            inner.x,
            inner.y + inner.height.saturating_sub(1),
            inner.width,
            1,
        );
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("esc", Style::new().fg(Color::Yellow).bold()),
                Span::raw(" close"),
            ])),
            hint_area,
        );
        return;
    }

    // Render target list
    let mut lines: Vec<Line> = Vec::new();
    for (i, (fmt, tool)) in app.convert_targets.iter().enumerate() {
        let marker = if i == app.convert_selected {
            "▶ "
        } else {
            "  "
        };
        let style = if i == app.convert_selected {
            Style::new().fg(Color::Yellow).bold()
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(
            format!("{marker}{fmt} [{tool}]"),
            style,
        )));
    }
    let list_area = Rect::new(inner.x, inner.y, inner.width, lines.len() as u16);
    frame.render_widget(Paragraph::new(lines), list_area);

    // Message below list
    if !app.convert_message.is_empty() {
        let msg_y = inner.y + list_area.height + 1;
        if msg_y < inner.y + inner.height {
            let msg_style = if app.convert_is_error {
                Style::new().fg(Color::Red)
            } else {
                Style::new().fg(Color::Green)
            };
            let msg_area = Rect::new(inner.x, msg_y, inner.width, 1);
            frame.render_widget(
                Paragraph::new(Span::styled(&app.convert_message, msg_style)),
                msg_area,
            );
        }
    }

    // Hint at bottom
    let hint_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(1),
        inner.width,
        1,
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("enter", Style::new().fg(Color::Yellow).bold()),
            Span::raw(" convert  "),
            Span::styled("esc", Style::new().fg(Color::Yellow).bold()),
            Span::raw(" cancel"),
        ])),
        hint_area,
    );
}

pub fn handle_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = Mode::Normal;
            app.convert_targets.clear();
            app.convert_message.clear();
        }
        KeyCode::Down | KeyCode::Char('s') => {
            if !app.convert_targets.is_empty() {
                app.convert_selected =
                    (app.convert_selected + 1) % app.convert_targets.len();
            }
        }
        KeyCode::Up | KeyCode::Char('w') => {
            if !app.convert_targets.is_empty() {
                app.convert_selected = if app.convert_selected == 0 {
                    app.convert_targets.len() - 1
                } else {
                    app.convert_selected - 1
                };
            }
        }
        KeyCode::Enter => run(app),
        _ => {}
    }
}

pub fn enter(app: &mut App) {
    let Some(idx) = app.table_state.selected() else {
        return;
    };
    let Some(book) = app.filtered_books.get(idx) else {
        return;
    };

    let (has_kepubify, has_ebook_convert) = converter::available_backends();
    if !has_kepubify && !has_ebook_convert {
        app.convert_message = "No conversion tools found (install kepubify or calibre's ebook-convert)".to_string();
        app.convert_is_error = true;
        app.convert_targets.clear();
        app.mode = Mode::Convert;
        return;
    }

    let targets = converter::target_formats(&book.formats, has_kepubify, has_ebook_convert);
    if targets.is_empty() {
        app.convert_message = "No formats to convert to".to_string();
        app.convert_is_error = true;
        app.convert_targets.clear();
        app.mode = Mode::Convert;
        return;
    }

    app.convert_targets = targets;
    app.convert_selected = 0;
    app.convert_message.clear();
    app.convert_is_error = false;
    app.mode = Mode::Convert;
}

pub fn run(app: &mut App) {
    if app.convert_targets.is_empty() {
        return;
    }

    let Some(idx) = app.table_state.selected() else {
        return;
    };
    let Some(book) = app.filtered_books.get(idx) else {
        return;
    };

    let (target, _tool) = app.convert_targets[app.convert_selected].clone();
    let book_path = book.path.clone();

    let Some(source_file) = converter::find_source_file(&book_path) else {
        app.convert_message = "No suitable source file found".to_string();
        app.convert_is_error = true;
        return;
    };

    match converter::convert(&book_path, &source_file, &target) {
        Ok(msg) => {
            let new_formats = scanner::scan_formats(&book_path);
            // Update filtered_books
            if let Some(fb) = app.filtered_books.get_mut(idx) {
                let book_path_clone = fb.path.clone();
                fb.formats = new_formats.clone();
                // Update matching entry in all_books
                if let Some(ab) = app.all_books.iter_mut().find(|b| b.path == book_path_clone)
                {
                    ab.formats = new_formats;
                }
            }
            app.convert_message = msg;
            app.convert_is_error = false;
            app.convert_targets.clear();
            app.mode = Mode::Normal;
        }
        Err(err) => {
            app.convert_message = err;
            app.convert_is_error = true;
        }
    }
}

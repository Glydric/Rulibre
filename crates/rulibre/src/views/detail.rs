use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use rulibre_core::metadata::Metadata;

use crate::app::{App, Focus, Mode};

pub(crate) struct DetailState {
    pub(crate) metadata: Option<Metadata>,
    pub(crate) scroll: u16,
    pub(crate) area: Rect,
}

impl Default for DetailState {
    fn default() -> Self {
        Self {
            metadata: None,
            scroll: 0,
            area: Rect::default(),
        }
    }
}

pub fn draw(app: &mut App, frame: &mut Frame, area: Rect) {
    app.detail.area = area;

    let border_style = if app.focus == Focus::Detail {
        Style::new().fg(Color::LightBlue)
    } else {
        Style::default()
    };

    let Some(meta) = &app.detail.metadata else {
        let block = Block::default()
            .title(" Detail ")
            .borders(Borders::ALL)
            .border_style(border_style);
        frame.render_widget(Paragraph::new("No metadata found.").block(block), area);
        return;
    };

    let key_style = Style::new().fg(Color::Yellow).bold();
    let section_style = Style::new()
        .fg(Color::Cyan)
        .bold()
        .add_modifier(Modifier::UNDERLINED);

    let mut lines: Vec<Line> = Vec::new();

    // ── Book Info section ──
    lines.push(Line::from(Span::styled("Book Info", section_style)));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("Title:    ", key_style),
        Span::raw(&meta.title),
    ]));

    if !meta.authors.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Author:   ", key_style),
            Span::raw(meta.authors.join(", ")),
        ]));
    }

    // Get formats from the selected book
    if let Some(idx) = app.table_state.selected()
        && let Some(book) = app.filtered_books.get(idx)
    {
        lines.push(Line::from(vec![
            Span::styled("Formats:  ", key_style),
            Span::raw(&book.formats),
        ]));
    }

    if !meta.series.is_empty() {
        let series_text = if meta.series_index.is_empty() {
            meta.series.clone()
        } else {
            format!("{} #{}", meta.series, meta.series_index)
        };
        lines.push(Line::from(vec![
            Span::styled("Series:   ", key_style),
            Span::raw(series_text),
        ]));
    }

    // ── Publishing section ──
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Publishing", section_style)));
    lines.push(Line::from(""));

    if !meta.publisher.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Publisher: ", key_style),
            Span::raw(&meta.publisher),
        ]));
    }
    if !meta.date.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Date:      ", key_style),
            Span::raw(&meta.date),
        ]));
    }
    if !meta.language.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Language:  ", key_style),
            Span::raw(&meta.language),
        ]));
    }
    for (scheme, value) in &meta.identifiers {
        lines.push(Line::from(vec![
            Span::styled(format!("{scheme}: "), key_style),
            Span::raw(value),
        ]));
    }
    if !meta.rating.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Rating:    ", key_style),
            Span::raw(&meta.rating),
        ]));
    }

    // ── Tags section ──
    if !meta.subjects.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Tags", section_style)));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::raw(meta.subjects.join(", "))));
    }

    // ── Description section ──
    if !meta.description.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Description", section_style)));
        lines.push(Line::from(""));
        // Word-wrap will be handled by Paragraph + Wrap
        lines.push(Line::from(Span::raw(&meta.description)));
    }

    // ── Unknown Metadata section (debug only) ──
    #[cfg(debug_assertions)]
    if !meta.unrecognized.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Unknown Metadata", section_style)));
        lines.push(Line::from(""));
        for tag in &meta.unrecognized {
            lines.push(Line::from(Span::raw(tag)));
        }
    }

    let block = Block::default()
        .title(" Detail ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.detail.scroll, 0));

    frame.render_widget(paragraph, area);
}

pub fn handle_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = Mode::Normal;
            app.detail.metadata = None;
            app.detail.scroll = 0;
            app.focus = Focus::Table;
        }
        KeyCode::Left | KeyCode::Char('a') => app.focus = Focus::Table,
        KeyCode::Right | KeyCode::Char('d') => app.focus = Focus::Detail,
        KeyCode::Char('c') => app.enter_convert(),
        KeyCode::Char('t') => app.send_to_device(),
        KeyCode::Down | KeyCode::Char('s') => match app.focus {
            Focus::Table => {
                app.next();
                app.open_detail();
            }
            Focus::Detail => {
                app.detail.scroll = app.detail.scroll.saturating_add(1);
            }
        },
        KeyCode::Up | KeyCode::Char('w') => match app.focus {
            Focus::Table => {
                app.previous();
                app.open_detail();
            }
            Focus::Detail => {
                app.detail.scroll = app.detail.scroll.saturating_sub(1);
            }
        },
        KeyCode::Enter => {
            if app.focus == Focus::Table {
                app.open_detail();
            } else {
                app.mode = Mode::Normal;
                app.detail.metadata = None;
                app.detail.scroll = 0;
                app.focus = Focus::Table;
            }
        }
        _ => {}
    }
}

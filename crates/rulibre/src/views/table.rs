use ratatui::{
    Frame,
    layout::{Constraint, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table},
};

use crate::app::{App, Focus, Mode};

pub fn draw(app: &mut App, frame: &mut Frame, area: Rect) {
    app.table_area = area;
    let header = Row::new(vec![
        Cell::from("Author").style(Style::new().bold()),
        Cell::from("Title").style(Style::new().bold()),
        Cell::from("Format").style(Style::new().bold()),
    ])
    .style(Style::new().fg(Color::Yellow))
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .filtered_books
        .iter()
        .map(|book| {
            Row::new(vec![
                Cell::from(book.author.as_str()),
                Cell::from(book.title.as_str()),
                Cell::from(book.formats.as_str()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(50),
        Constraint::Percentage(20),
    ];

    let title = if app.search_query.is_empty() {
        format!(" Rulibre — {} books ", app.filtered_books.len())
    } else {
        format!(
            " Rulibre — {} / {} books ",
            app.filtered_books.len(),
            app.all_books.len()
        )
    };

    let border_style = if matches!(app.mode, Mode::Detail) && app.focus == Focus::Table {
        Style::new().fg(Color::LightBlue)
    } else {
        Style::default()
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .row_highlight_style(
            Style::new()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(table, area, &mut app.table_state);

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut app.scrollbar_state,
    );
}

fn send_hint_spans() -> Vec<Span<'static>> {
    vec![
        Span::styled("t", Style::new().fg(Color::Yellow).bold()),
        Span::raw(" send  "),
    ]
}

pub fn draw_status_bar(app: &App, frame: &mut Frame, area: Rect) {
    let has_device = app.device.connected.is_some();

    let spans = match app.mode {
        Mode::Setup => vec![Span::raw("")],
        Mode::Search => vec![
            Span::styled(" /", Style::new().fg(Color::Yellow).bold()),
            Span::raw(&app.search_query),
            Span::styled("█", Style::new().fg(Color::Yellow)),
        ],
        Mode::Detail => {
            let mut s = vec![
                Span::styled(" ←", Style::new().fg(Color::Yellow).bold()),
                Span::raw("/"),
                Span::styled("→", Style::new().fg(Color::Yellow).bold()),
                Span::raw(" focus  "),
                Span::styled("w", Style::new().fg(Color::Yellow).bold()),
                Span::raw("/"),
                Span::styled("↑", Style::new().fg(Color::Yellow).bold()),
                Span::raw(" up  "),
                Span::styled("s", Style::new().fg(Color::Yellow).bold()),
                Span::raw("/"),
                Span::styled("↓", Style::new().fg(Color::Yellow).bold()),
                Span::raw(" down  "),
                Span::styled("c", Style::new().fg(Color::Yellow).bold()),
                Span::raw(" convert  "),
            ];
            if has_device {
                s.extend(send_hint_spans());
            }
            s.push(Span::styled("q", Style::new().fg(Color::Yellow).bold()));
            s.push(Span::raw("/"));
            s.push(Span::styled("esc", Style::new().fg(Color::Yellow).bold()));
            s.push(Span::raw(" close"));
            s
        }
        Mode::Convert => vec![
            Span::styled(" w", Style::new().fg(Color::Yellow).bold()),
            Span::raw("/"),
            Span::styled("↑", Style::new().fg(Color::Yellow).bold()),
            Span::raw(" up  "),
            Span::styled("s", Style::new().fg(Color::Yellow).bold()),
            Span::raw("/"),
            Span::styled("↓", Style::new().fg(Color::Yellow).bold()),
            Span::raw(" down  "),
            Span::styled("enter", Style::new().fg(Color::Yellow).bold()),
            Span::raw(" convert  "),
            Span::styled("esc", Style::new().fg(Color::Yellow).bold()),
            Span::raw(" cancel"),
        ],
        Mode::Normal => {
            let mut s = vec![
                Span::styled(" w", Style::new().fg(Color::Yellow).bold()),
                Span::raw("/"),
                Span::styled("↑", Style::new().fg(Color::Yellow).bold()),
                Span::raw(" up  "),
                Span::styled("s", Style::new().fg(Color::Yellow).bold()),
                Span::raw("/"),
                Span::styled("↓", Style::new().fg(Color::Yellow).bold()),
                Span::raw(" down  "),
                Span::styled("enter", Style::new().fg(Color::Yellow).bold()),
                Span::raw(" detail  "),
                Span::styled("c", Style::new().fg(Color::Yellow).bold()),
                Span::raw(" convert  "),
            ];
            if has_device {
                s.extend(send_hint_spans());
            }
            s.push(Span::styled("/", Style::new().fg(Color::Yellow).bold()));
            s.push(Span::raw(" search  "));
            s.push(Span::styled("q", Style::new().fg(Color::Yellow).bold()));
            s.push(Span::raw("/"));
            s.push(Span::styled("esc", Style::new().fg(Color::Yellow).bold()));
            s.push(Span::raw(" quit"));
            s
        }
    };

    let style = match app.mode {
        Mode::Search => Style::new().bg(Color::DarkGray),
        _ => Style::new().bg(Color::Black),
    };

    frame.render_widget(Paragraph::new(Line::from(spans)).style(style), area);
}

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, TableState, Wrap,
    },
};

use rulibre::metadata::{self, Metadata};
use rulibre::scanner::Book;

enum Mode {
    Normal,
    Search,
    Detail,
}

#[derive(PartialEq)]
enum Focus {
    Table,
    Detail,
}

pub struct App {
    all_books: Vec<Book>,
    filtered_books: Vec<Book>,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    mode: Mode,
    search_query: String,
    detail: Option<Metadata>,
    detail_scroll: u16,
    focus: Focus,
    table_area: Rect,
    detail_area: Rect,
}

impl App {
    pub fn new(books: Vec<Book>) -> Self {
        let len = books.len();
        let mut table_state = TableState::default();
        if !books.is_empty() {
            table_state.select(Some(0));
        }
        Self {
            filtered_books: books.clone(),
            all_books: books,
            table_state,
            scrollbar_state: ScrollbarState::new(len.saturating_sub(1)),
            mode: Mode::Normal,
            search_query: String::new(),
            detail: None,
            detail_scroll: 0,
            focus: Focus::Table,
            table_area: Rect::default(),
            detail_area: Rect::default(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if self.handle_key(key.code) {
                        return Ok(());
                    }
                }
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        self.handle_click(mouse.column, mouse.row);
                    }
                    MouseEventKind::ScrollUp => {
                        self.handle_scroll(mouse.column, mouse.row, true);
                    }
                    MouseEventKind::ScrollDown => {
                        self.handle_scroll(mouse.column, mouse.row, false);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    /// Returns `true` if the app should quit.
    fn handle_key(&mut self, code: KeyCode) -> bool {
        match self.mode {
            Mode::Normal => match code {
                KeyCode::Char('q') | KeyCode::Esc => return true,
                KeyCode::Down | KeyCode::Char('s') => self.next(),
                KeyCode::Up | KeyCode::Char('w') => self.previous(),
                KeyCode::Char('/') => {
                    self.mode = Mode::Search;
                    self.search_query.clear();
                }
                KeyCode::Enter => self.open_detail(),
                _ => {}
            },
            Mode::Search => match code {
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                    self.search_query.clear();
                    self.apply_filter();
                }
                KeyCode::Enter => {
                    self.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.apply_filter();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.apply_filter();
                }
                _ => {}
            },
            Mode::Detail => match code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.mode = Mode::Normal;
                    self.detail = None;
                    self.detail_scroll = 0;
                    self.focus = Focus::Table;
                }
                KeyCode::Left | KeyCode::Char('a') => self.focus = Focus::Table,
                KeyCode::Right | KeyCode::Char('d') => self.focus = Focus::Detail,
                KeyCode::Down | KeyCode::Char('s') => match self.focus {
                    Focus::Table => self.next(),
                    Focus::Detail => {
                        self.detail_scroll = self.detail_scroll.saturating_add(1);
                    }
                },
                KeyCode::Up | KeyCode::Char('w') => match self.focus {
                    Focus::Table => self.previous(),
                    Focus::Detail => {
                        self.detail_scroll = self.detail_scroll.saturating_sub(1);
                    }
                },
                KeyCode::Enter => {
                    if self.focus == Focus::Table {
                        self.open_detail();
                    } else {
                        self.mode = Mode::Normal;
                        self.detail = None;
                        self.detail_scroll = 0;
                        self.focus = Focus::Table;
                    }
                }
                _ => {}
            },
        }
        false
    }

    fn open_detail(&mut self) {
        let Some(idx) = self.table_state.selected() else {
            return;
        };
        let Some(book) = self.filtered_books.get(idx) else {
            return;
        };
        self.detail = metadata::parse_opf(&book.path);
        self.detail_scroll = 0;
        self.mode = Mode::Detail;
    }

    fn handle_click(&mut self, col: u16, row: u16) {
        if Self::is_in_area(col, row, self.table_area) {
            self.focus = Focus::Table;

            let area = self.table_area;
            // Account for border (1) + header (1) + header bottom_margin (1) = 3 rows offset
            let content_start = area.y + 3;
            if row < content_start {
                return;
            }

            let clicked_row = (row - content_start) as usize;
            let offset = self.table_state.offset();
            let idx = offset + clicked_row;

            if idx < self.filtered_books.len() {
                self.table_state.select(Some(idx));
                self.scrollbar_state = self.scrollbar_state.position(idx);
                self.open_detail();
            }
        } else if matches!(self.mode, Mode::Detail)
            && Self::is_in_area(col, row, self.detail_area)
        {
            self.focus = Focus::Detail;
        }
    }

    fn is_in_area(col: u16, row: u16, area: Rect) -> bool {
        col >= area.x && col < area.x + area.width && row >= area.y && row < area.y + area.height
    }

    fn handle_scroll(&mut self, col: u16, row: u16, up: bool) {
        if Self::is_in_area(col, row, self.table_area) {
            if up {
                self.previous();
            } else {
                self.next();
            }
        } else if matches!(self.mode, Mode::Detail)
            && Self::is_in_area(col, row, self.detail_area)
        {
            if up {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
            } else {
                self.detail_scroll = self.detail_scroll.saturating_add(1);
            }
        }
    }

    fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_books = self.all_books.clone();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_books = self
                .all_books
                .iter()
                .filter(|b| {
                    b.author.to_lowercase().contains(&query)
                        || b.title.to_lowercase().contains(&query)
                        || b.formats.to_lowercase().contains(&query)
                })
                .cloned()
                .collect();
        }
        self.scrollbar_state = ScrollbarState::new(self.filtered_books.len().saturating_sub(1));
        if self.filtered_books.is_empty() {
            self.table_state.select(None);
        } else {
            self.table_state.select(Some(0));
            self.scrollbar_state = self.scrollbar_state.position(0);
        }
    }

    fn next(&mut self) {
        if self.filtered_books.is_empty() {
            return;
        }
        let i = self.table_state.selected().map_or(0, |i| {
            if i >= self.filtered_books.len() - 1 {
                0
            } else {
                i + 1
            }
        });
        self.table_state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }

    fn previous(&mut self) {
        if self.filtered_books.is_empty() {
            return;
        }
        let i = self.table_state.selected().map_or(0, |i| {
            if i == 0 {
                self.filtered_books.len() - 1
            } else {
                i - 1
            }
        });
        self.table_state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }

    fn draw(&mut self, frame: &mut Frame) {
        let chunks =
            Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(frame.area());

        match self.mode {
            Mode::Detail => {
                let cols =
                    Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
                        .split(chunks[0]);
                self.draw_table(frame, cols[0]);
                self.draw_detail(frame, cols[1]);
            }
            _ => {
                self.draw_table(frame, chunks[0]);
            }
        }

        self.draw_status_bar(frame, chunks[1]);
    }

    fn draw_table(&mut self, frame: &mut Frame, area: Rect) {
        self.table_area = area;
        let header = Row::new(vec![
            Cell::from("Author").style(Style::new().bold()),
            Cell::from("Title").style(Style::new().bold()),
            Cell::from("Format").style(Style::new().bold()),
        ])
        .style(Style::new().fg(Color::Yellow))
        .height(1)
        .bottom_margin(1);

        let rows: Vec<Row> = self
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

        let title = if self.search_query.is_empty() {
            format!(" Rulibre — {} books ", self.filtered_books.len())
        } else {
            format!(
                " Rulibre — {} / {} books ",
                self.filtered_books.len(),
                self.all_books.len()
            )
        };

        let border_style = if matches!(self.mode, Mode::Detail) && self.focus == Focus::Table {
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

        frame.render_stateful_widget(table, area, &mut self.table_state);

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.scrollbar_state,
        );
    }

    fn draw_detail(&mut self, frame: &mut Frame, area: Rect) {
        self.detail_area = area;

        let border_style = if self.focus == Focus::Detail {
            Style::new().fg(Color::LightBlue)
        } else {
            Style::default()
        };

        let Some(meta) = &self.detail else {
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
        if let Some(idx) = self.table_state.selected()
            && let Some(book) = self.filtered_books.get(idx)
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
            .scroll((self.detail_scroll, 0));

        frame.render_widget(paragraph, area);
    }

    fn draw_status_bar(&self, frame: &mut Frame, area: Rect) {
        let bar = match self.mode {
            Mode::Search => Line::from(vec![
                Span::styled(" /", Style::new().fg(Color::Yellow).bold()),
                Span::raw(&self.search_query),
                Span::styled("█", Style::new().fg(Color::Yellow)),
            ]),
            Mode::Detail => Line::from(vec![
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
                Span::styled("q", Style::new().fg(Color::Yellow).bold()),
                Span::raw("/"),
                Span::styled("esc", Style::new().fg(Color::Yellow).bold()),
                Span::raw(" close"),
            ]),
            Mode::Normal => Line::from(vec![
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
                Span::styled("/", Style::new().fg(Color::Yellow).bold()),
                Span::raw(" search  "),
                Span::styled("q", Style::new().fg(Color::Yellow).bold()),
                Span::raw("/"),
                Span::styled("esc", Style::new().fg(Color::Yellow).bold()),
                Span::raw(" quit"),
            ]),
        };

        let style = match self.mode {
            Mode::Search => Style::new().bg(Color::DarkGray),
            _ => Style::new().bg(Color::Black),
        };

        frame.render_widget(Paragraph::new(bar).style(style), area);
    }
}

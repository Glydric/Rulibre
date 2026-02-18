use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, TableState,
    },
};

use crate::scanner::Book;

enum Mode {
    Normal,
    Search,
}

pub struct App {
    all_books: Vec<Book>,
    filtered_books: Vec<Book>,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    mode: Mode,
    search_query: String,
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
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match self.mode {
                    Mode::Normal => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Down | KeyCode::Char('s') => self.next(),
                        KeyCode::Up | KeyCode::Char('w') => self.previous(),
                        KeyCode::Char('/') => {
                            self.mode = Mode::Search;
                            self.search_query.clear();
                        }
                        _ => {}
                    },
                    Mode::Search => match key.code {
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
                }
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
        self.scrollbar_state =
            ScrollbarState::new(self.filtered_books.len().saturating_sub(1));
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
            if i >= self.filtered_books.len() - 1 { 0 } else { i + 1 }
        });
        self.table_state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }

    fn previous(&mut self) {
        if self.filtered_books.is_empty() {
            return;
        }
        let i = self.table_state.selected().map_or(0, |i| {
            if i == 0 { self.filtered_books.len() - 1 } else { i - 1 }
        });
        self.table_state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }

    fn draw(&mut self, frame: &mut Frame) {
        let chunks = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

        self.draw_table(frame, chunks[0]);
        self.draw_status_bar(frame, chunks[1]);
    }

    fn draw_table(&mut self, frame: &mut Frame, area: Rect) {
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
            format!(" rulibre — {} books ", self.filtered_books.len())
        } else {
            format!(
                " rulibre — {} / {} books ",
                self.filtered_books.len(),
                self.all_books.len()
            )
        };

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL),
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

    fn draw_status_bar(&self, frame: &mut Frame, area: Rect) {
        let bar = match self.mode {
            Mode::Search => Line::from(vec![
                Span::styled(" /", Style::new().fg(Color::Yellow).bold()),
                Span::raw(&self.search_query),
                Span::styled("█", Style::new().fg(Color::Yellow)),
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
            Mode::Normal => Style::new().bg(Color::Black),
        };

        frame.render_widget(Paragraph::new(bar).style(style), area);
    }
}

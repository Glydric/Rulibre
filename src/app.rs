use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Margin},
    style::{Color, Modifier, Style, Stylize},
    widgets::{
        Block, Borders, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};

use crate::scanner::Book;

pub struct App {
    books: Vec<Book>,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
}

impl App {
    pub fn new(books: Vec<Book>) -> Self {
        let len = books.len();
        let mut table_state = TableState::default();
        if !books.is_empty() {
            table_state.select(Some(0));
        }
        Self {
            books,
            table_state,
            scrollbar_state: ScrollbarState::new(len.saturating_sub(1)),
        }
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Down | KeyCode::Char('s') => self.next(),
                    KeyCode::Up | KeyCode::Char('w') => self.previous(),
                    _ => {}
                }
            }
        }
    }

    fn next(&mut self) {
        if self.books.is_empty() {
            return;
        }
        let i = self
            .table_state
            .selected()
            .map_or(0, |i| if i >= self.books.len() - 1 { 0 } else { i + 1 });
        self.table_state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }

    fn previous(&mut self) {
        if self.books.is_empty() {
            return;
        }
        let i = self
            .table_state
            .selected()
            .map_or(0, |i| if i == 0 { self.books.len() - 1 } else { i - 1 });
        self.table_state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let header = Row::new(vec![
            Cell::from("Author").style(Style::new().bold()),
            Cell::from("Title").style(Style::new().bold()),
            Cell::from("Format").style(Style::new().bold()),
        ])
        .style(Style::new().fg(Color::Yellow))
        .height(1)
        .bottom_margin(1);

        let rows: Vec<Row> = self
            .books
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

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title(format!(" rulibre — {} books ", self.books.len()))
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
}

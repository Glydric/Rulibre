use std::io;
use std::sync::mpsc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    widgets::{ScrollbarState, TableState},
};

use rulibre::config;
use rulibre::device::{DeviceEvent, DeviceState};
use rulibre::metadata;
use rulibre::scanner::{self, Book};

use crate::views;
use crate::views::convert::ConvertState;
use crate::views::detail::DetailState;
use crate::views::notification::NotificationState;
use crate::views::setup::SetupState;

#[derive(PartialEq)]
pub(crate) enum Mode {
    Setup,
    Normal,
    Search,
    Detail,
    Convert,
}

#[derive(PartialEq)]
pub(crate) enum Focus {
    Table,
    Detail,
}

pub struct App {
    pub(crate) all_books: Vec<Book>,
    pub(crate) filtered_books: Vec<Book>,
    pub(crate) table_state: TableState,
    pub(crate) scrollbar_state: ScrollbarState,
    pub(crate) mode: Mode,
    pub(crate) search_query: String,
    pub(crate) focus: Focus,
    pub(crate) table_area: Rect,
    pub(crate) setup: SetupState,
    pub(crate) detail: DetailState,
    pub(crate) convert: ConvertState,
    pub(crate) notification: NotificationState,
    pub(crate) device: DeviceState,
}

impl App {
    pub fn new(cfg: Option<config::Config>) -> Self {
        match cfg {
            Some(cfg) if config::is_calibre_library(std::path::Path::new(&cfg.library_path)) => {
                let books = scanner::scan_library(std::path::Path::new(&cfg.library_path));
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
                    focus: Focus::Table,
                    table_area: Rect::default(),
                    setup: SetupState::default(),
                    detail: DetailState::default(),
                    convert: ConvertState::default(),
                    notification: NotificationState::default(),
                    device: DeviceState::default(),
                }
            }
            _ => Self {
                all_books: Vec::new(),
                filtered_books: Vec::new(),
                table_state: TableState::default(),
                scrollbar_state: ScrollbarState::new(0),
                mode: Mode::Setup,
                search_query: String::new(),
                focus: Focus::Table,
                table_area: Rect::default(),
                setup: SetupState::default(),
                detail: DetailState::default(),
                convert: ConvertState::default(),
                notification: NotificationState::default(),
                device: DeviceState::default(),
            },
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        device_rx: mpsc::Receiver<DeviceEvent>,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            // Process device events from background thread
            while let Ok(ev) = device_rx.try_recv() {
                let msg = self.device.handle_event(ev);
                self.notification.set(Ok(msg));
            }

            // Tick notification auto-clear (~3s at 250ms poll)
            self.notification.tick();

            // Poll with timeout so we can process device events between frames
            if !event::poll(Duration::from_millis(250))? {
                continue;
            }

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
                KeyCode::Char('c') => views::convert::enter(self),
                KeyCode::Char('t') => self.send_to_device(),
                _ => {}
            },
            Mode::Setup => return views::setup::handle_key(self, code),
            Mode::Search => views::search::handle_key(self, code),
            Mode::Convert => views::convert::handle_key(self, code),
            Mode::Detail => views::detail::handle_key(self, code),
        }
        false
    }

    pub(crate) fn open_detail(&mut self) {
        let Some(idx) = self.table_state.selected() else {
            return;
        };
        let Some(book) = self.filtered_books.get(idx) else {
            return;
        };
        self.detail.metadata = metadata::parse_opf(&book.path);
        self.detail.scroll = 0;
        self.mode = Mode::Detail;
    }

    pub(crate) fn enter_convert(&mut self) {
        views::convert::enter(self);
    }

    pub(crate) fn send_to_device(&mut self) {
        let Some(idx) = self.table_state.selected() else {
            return;
        };
        let Some(book) = self.filtered_books.get(idx) else {
            return;
        };
        let result = self.device.send_book(&book.path.clone(), &book.author);
        self.notification.set(result);
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
            && Self::is_in_area(col, row, self.detail.area)
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
            && Self::is_in_area(col, row, self.detail.area)
        {
            if up {
                self.detail.scroll = self.detail.scroll.saturating_sub(1);
            } else {
                self.detail.scroll = self.detail.scroll.saturating_add(1);
            }
        }
    }

    pub(crate) fn apply_filter(&mut self) {
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

    pub(crate) fn next(&mut self) {
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

    pub(crate) fn previous(&mut self) {
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
                views::table::draw(self, frame, cols[0]);
                views::detail::draw(self, frame, cols[1]);
            }
            Mode::Setup => {
                views::setup::draw(self, frame);
                return;
            }
            Mode::Convert => {
                views::table::draw(self, frame, chunks[0]);
                views::convert::draw(self, frame);
            }
            _ => {
                views::table::draw(self, frame, chunks[0]);
            }
        }

        views::table::draw_status_bar(self, frame, chunks[1]);
        views::notification::draw(self, frame);
    }
}

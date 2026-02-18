mod app;
mod config;
mod metadata;
mod scanner;

use std::{
    io::{self, stdout},
    path::Path,
    process,
};

use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

fn main() -> io::Result<()> {
    let cfg = match config::load_config() {
        Some(c) => c,
        None => {
            let path = config::prompt_library_path();
            let c = config::Config { library_path: path };
            config::save_config(&c);
            eprintln!("Config saved to {}", config::config_path().display());
            c
        }
    };

    let library_path = Path::new(&cfg.library_path);
    if !library_path.is_dir() {
        eprintln!("Library path does not exist: {}", cfg.library_path);
        process::exit(1);
    }
    if !config::is_calibre_library(library_path) {
        eprintln!(
            "Not a valid Calibre library (missing metadata.db): {}",
            cfg.library_path
        );
        process::exit(1);
    }

    let books = scanner::scan_library(library_path);
    let mut app = app::App::new(books);

    // todo to understand
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = ratatui::init();

    let result = app.run(&mut terminal);

    ratatui::restore();
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

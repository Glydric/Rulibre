mod app;
mod views;

use std::{
    io::{self, stdout},
    process::Command,
};

use crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rulibre::config;

fn main() -> io::Result<()> {
    if std::env::args().any(|a| a == "--config") {
        let config_path = config::Config::path();
        std::fs::create_dir_all(config_path.parent().unwrap())?;
        if !config_path.exists() {
            // Create a default config so the user has something to edit
            let default = config::Config {
                library_path: String::new(),
            };
            default.save();
        }
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        let status = Command::new(&editor).arg(&config_path).status()?;
        std::process::exit(status.code().unwrap_or(0));
    }

    let cfg = config::Config::load();
    let mut app = app::App::new(cfg);

    // used to have complete control of terminal text
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    let mut terminal = ratatui::init();

    let result = app.run(&mut terminal);

    ratatui::restore();
    stdout().execute(DisableMouseCapture)?;
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

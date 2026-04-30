mod app;
mod views;

use std::{io::stdout, sync::mpsc, thread, time::Duration};

use crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use rulibre_core::config;
use rulibre_core::device::{self, DeviceEvent};

fn main() -> std::io::Result<()> {
    if std::env::args().any(|a| a == "--config") {
        return open_config_in_editor();
    }
    run()
}

fn run() -> std::io::Result<()> {
    let cfg = config::Config::load();
    let mut app = app::App::new(cfg);

    // Spawn background device detection thread
    let (device_sender, device_receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut was_connected = false;
        loop {
            let detected = device::detect_device();
            match (&detected, was_connected) {
                (Some(dev), false) => {
                    let _ = device_sender.send(DeviceEvent::Connected(dev.clone()));
                    was_connected = true;
                }
                (None, true) => {
                    let _ = device_sender.send(DeviceEvent::Disconnected);
                    was_connected = false;
                }
                _ => {}
            }
            thread::sleep(Duration::from_secs(2));
        }
    });

    // used to have complete control of terminal text
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    let mut terminal = ratatui::init();

    let result = app.run(&mut terminal, device_receiver);

    ratatui::restore();
    stdout().execute(DisableMouseCapture)?;
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

fn open_config_in_editor() -> std::io::Result<()> {
    let config_path = config::Config::path();
    std::fs::create_dir_all(config_path.parent().unwrap())?;
    if !config_path.exists() {
        let default = config::Config {
            library_path: String::new(),
        };
        default.save();
    }
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status()?;
    std::process::exit(status.code().unwrap_or(0));
}

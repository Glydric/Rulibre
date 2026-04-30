#![cfg_attr(
    all(not(debug_assertions), not(target_arch = "wasm32")),
    windows_subsystem = "windows"
)]

#[cfg(not(target_arch = "wasm32"))]
mod commands;
#[cfg(not(target_arch = "wasm32"))]
mod device_watch;

#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
mod components;
#[cfg(target_arch = "wasm32")]
mod ipc;
#[cfg(target_arch = "wasm32")]
mod keys;
#[cfg(target_arch = "wasm32")]
mod state;
#[cfg(target_arch = "wasm32")]
mod types;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct DeviceWatchState(
    pub std::sync::Arc<std::sync::Mutex<Option<rulibre_core::device::Device>>>,
);

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::io::Result<()> {
    if std::env::args().any(|a| a == "--config") {
        return open_config_in_editor();
    }
    run();
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn run() {
    use tauri::Manager;
    tauri::Builder::default()
        .manage(DeviceWatchState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            let shared: tauri::State<DeviceWatchState> = app.state();
            let shared = shared.0.clone();
            tauri::async_runtime::spawn(device_watch::watch_loop(handle, shared));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_config,
            commands::save_config,
            commands::validate_library,
            commands::scan_library,
            commands::parse_metadata,
            commands::available_backends,
            commands::target_formats,
            commands::convert_book,
            commands::send_to_device,
            commands::current_device,
        ])
        .run(tauri::generate_context!())
        .expect("error while running rulibre GUI");
}

#[cfg(not(target_arch = "wasm32"))]
fn open_config_in_editor() -> std::io::Result<()> {
    use rulibre_core::config::Config;
    let config_path = Config::path();
    std::fs::create_dir_all(config_path.parent().unwrap())?;
    if !config_path.exists() {
        Config {
            library_path: String::new(),
        }
        .save();
    }
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status()?;
    std::process::exit(status.code().unwrap_or(0));
}

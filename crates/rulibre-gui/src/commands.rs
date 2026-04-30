use std::path::{Path, PathBuf};

use rulibre_core::{
    config, converter,
    device::{Device, DeviceState},
    metadata::{self, Metadata},
    scanner::{self, Book},
};
use serde::Serialize;

use super::DeviceWatchState;

#[tauri::command]
pub fn load_config() -> Option<config::Config> {
    config::Config::load()
}

#[tauri::command]
pub fn save_config(library_path: String) -> Result<(), String> {
    config::Config { library_path }.save();
    Ok(())
}

#[tauri::command]
pub fn validate_library(path: String) -> Result<String, String> {
    let sanitized = config::sanitize_path(&path);
    if sanitized.is_empty() {
        return Err("No path provided.".into());
    }
    let p = Path::new(&sanitized);
    if !p.is_dir() {
        return Err(format!("Path does not exist: {sanitized}"));
    }
    if !config::is_calibre_library(p) {
        return Err("Not a valid Calibre library (missing metadata.db).".into());
    }
    Ok(sanitized)
}

#[tauri::command]
pub fn scan_library(path: String) -> Vec<Book> {
    scanner::scan_library(Path::new(&path))
}

#[tauri::command]
pub fn parse_metadata(book_path: String) -> Option<Metadata> {
    metadata::parse_opf(Path::new(&book_path))
}

#[tauri::command]
pub fn available_backends() -> (bool, bool) {
    converter::available_backends()
}

#[tauri::command]
pub fn target_formats(
    formats: String,
    has_kepubify: bool,
    has_calibre_convert: bool,
) -> Vec<(String, String)> {
    converter::target_formats(&formats, has_kepubify, has_calibre_convert)
}

#[derive(Serialize)]
pub struct ConvertOk {
    pub message: String,
    pub new_formats: String,
}

#[tauri::command]
pub fn convert_book(book_path: String, target: String) -> Result<ConvertOk, String> {
    let book_path = PathBuf::from(book_path);
    let source = converter::find_source_file(&book_path)
        .ok_or_else(|| "No suitable source file found".to_string())?;
    let message = converter::convert(&book_path, &source, &target)?;
    let new_formats = scanner::scan_formats(&book_path);
    Ok(ConvertOk {
        message,
        new_formats,
    })
}

#[tauri::command]
pub fn send_to_device(
    book_path: String,
    author: String,
    state: tauri::State<'_, DeviceWatchState>,
) -> Result<String, String> {
    let device = state.0.lock().unwrap().clone();
    let s = DeviceState { connected: device };
    s.send_book(&PathBuf::from(book_path), &author)
}

#[tauri::command]
pub fn current_device(state: tauri::State<'_, DeviceWatchState>) -> Option<Device> {
    state.0.lock().unwrap().clone()
}

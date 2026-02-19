use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::converter;

#[derive(Clone, Debug)]
pub enum DeviceKind {
    Kobo,
    Kindle,
}

#[derive(Clone, Debug)]
pub struct Device {
    pub kind: DeviceKind,
    pub mount_point: PathBuf,
}

pub enum DeviceEvent {
    Connected(Device),
    Disconnected,
}

pub struct DeviceState {
    pub connected: Option<Device>,
}

impl Default for DeviceState {
    fn default() -> Self {
        Self { connected: None }
    }
}

impl DeviceState {
    /// Process a device event, returning a status message.
    pub fn handle_event(&mut self, event: DeviceEvent) -> String {
        match event {
            DeviceEvent::Connected(dev) => {
                let msg = format!("{} connected", dev.name());
                self.connected = Some(dev);
                msg
            }
            DeviceEvent::Disconnected => {
                self.connected = None;
                "Device disconnected".to_string()
            }
        }
    }

    /// Find or convert a compatible file, then copy it to the device.
    pub fn send_book(&self, book_path: &Path, author: &str) -> Result<String, String> {
        let dev = self
            .connected
            .as_ref()
            .ok_or_else(|| "No device connected".to_string())?;

        let file = prepare_file(book_path, dev)?;
        send_to_device(&file, dev, author)
    }
}

impl Device {
    pub fn books_dir(&self, author: &str) -> PathBuf {
        match self.kind {
            DeviceKind::Kobo => self.mount_point.join(author),
            DeviceKind::Kindle => self.mount_point.join("documents"),
        }
    }

    pub fn supported_extensions(&self) -> &[&str] {
        match self.kind {
            DeviceKind::Kobo => &["kepub.epub", "epub"],
            DeviceKind::Kindle => &["azw3", "mobi"],
        }
    }

    pub fn name(&self) -> &str {
        match self.kind {
            DeviceKind::Kobo => "Kobo",
            DeviceKind::Kindle => "Kindle",
        }
    }
}

/// Check known mount points for connected e-readers.
pub fn detect_device() -> Option<Device> {
    // todo update to be less system dependent
    let kobo = Path::new("/Volumes/KOBOeReader");
    if kobo.is_dir() {
        return Some(Device {
            kind: DeviceKind::Kobo,
            mount_point: kobo.to_path_buf(),
        });
    }

    // todo update to be less system dependent
    let kindle = Path::new("/Volumes/Kindle");
    if kindle.is_dir() {
        return Some(Device {
            kind: DeviceKind::Kindle,
            mount_point: kindle.to_path_buf(),
        });
    }

    None
}

/// Find or create a compatible file for the device.
/// For Kobo: prefer kepub, convert from epub if needed, create epub first if missing.
fn prepare_file(book_path: &Path, device: &Device) -> Result<PathBuf, String> {
    // extract file or return error
    let file = find_file_with_ext(book_path, device.supported_extensions())
        .ok_or(format!("No compatible format for {}", device.name()))?;

    let file_name = file.to_string_lossy().to_string();

    // if the target is kobo and we have a .epub (without .kepub.epub first) then try to convert to kepub
    if matches!(device.kind, DeviceKind::Kobo)
        && !file_name.ends_with(".kepub.epub")
        && file_name.ends_with(".epub")
    {
        // Kobo: try to produce a kepub
        let epub = ensure_epub(book_path)?;
        converter::convert(book_path, &epub, "KEPUB")?;

        return find_file_with_ext(book_path, &["kepub.epub"])
            .ok_or_else(|| "KEPUB conversion produced no file".to_string());
    }
    return Ok(file);
}

/// Look for a file matching one of the given extensions (in priority order).
fn find_file_with_ext(book_path: &Path, extensions: &[&str]) -> Option<PathBuf> {
    let entries = fs::read_dir(book_path).ok()?;
    let files: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| e.path())
        .collect();

    for ext in extensions {
        let suffix = format!(".{ext}");
        if let Some(path) = files
            .iter()
            .find(|p| p.to_string_lossy().to_lowercase().ends_with(&suffix))
        {
            return Some(path.clone());
        }
    }
    None
}

/// Ensure an epub exists in the book directory, converting from another format if needed.
fn ensure_epub(book_path: &Path) -> Result<PathBuf, String> {
    if let Some(epub) = find_file_with_ext(book_path, &["epub"]) {
        return Ok(epub);
    }

    let source = converter::find_source_file(book_path)
        .ok_or_else(|| "No source file to convert from".to_string())?;

    converter::convert(book_path, &source, "EPUB")?;

    find_file_with_ext(book_path, &["epub"])
        .ok_or_else(|| "EPUB conversion produced no file".to_string())
}

/// Copy a file to the device's books directory.
fn send_to_device(file: &Path, device: &Device, author: &str) -> Result<String, String> {
    let dest_dir = device.books_dir(author);

    if !dest_dir.is_dir() {
        fs::create_dir_all(&dest_dir).map_err(|e| format!("Failed to create dir: {e}"))?;
    }

    let file_name = file
        .file_name()
        .ok_or_else(|| "Invalid file name".to_string())?;
    let dest = dest_dir.join(file_name);

    fs::copy(file, &dest).map_err(|e| format!("Copy failed: {e}"))?;

    Ok(format!(
        "Sent {} to {}",
        file_name.to_string_lossy(),
        device.name()
    ))
}

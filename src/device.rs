use std::{
    fs,
    path::{Path, PathBuf},
};

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

    /// Find a compatible file in the book directory and copy it to the device.
    pub fn send_book(&self, book_path: &Path) -> Result<String, String> {
        let dev = self
            .connected
            .as_ref()
            .ok_or_else(|| "No device connected".to_string())?;

        let file = find_compatible_file(book_path, dev)
            .ok_or_else(|| format!("No compatible format for {}", dev.name()))?;

        send_to_device(&file, dev)
    }
}

impl Device {
    pub fn books_dir(&self) -> PathBuf {
        match self.kind {
            // for kobo just use root
            DeviceKind::Kobo => self.mount_point.clone(),
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

/// Find the best compatible file in a book directory for the given device.
fn find_compatible_file(book_path: &Path, device: &Device) -> Option<PathBuf> {
    let entries = fs::read_dir(book_path).ok()?;

    let files: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| e.path())
        .collect();

    for ext in device.supported_extensions() {
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

/// Copy a file to the device's books directory.
fn send_to_device(file: &Path, device: &Device) -> Result<String, String> {
    let dest_dir = device.books_dir();

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

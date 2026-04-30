use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Book {
    pub author: String,
    pub title: String,
    pub formats: String,
    pub path: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub publisher: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub subjects: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub identifiers: Vec<(String, String)>,
    #[serde(default)]
    pub rating: String,
    #[serde(default)]
    pub series: String,
    #[serde(default)]
    pub series_index: String,
    #[serde(default)]
    pub unrecognized: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DeviceKind {
    Kobo,
    Kindle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Device {
    pub kind: DeviceKind,
    pub mount_point: String,
}

impl Device {
    pub fn name(&self) -> &'static str {
        match self.kind {
            DeviceKind::Kobo => "Kobo",
            DeviceKind::Kindle => "Kindle",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DeviceEvent {
    Connected(Device),
    Disconnected,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub library_path: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConvertOk {
    pub message: String,
    pub new_formats: String,
}

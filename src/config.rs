use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub library_path: String,
}

impl Config {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .expect("could not determine config directory")
            .join("dev.miglio.rulibre")
            .join("config.toml")
    }

    pub fn load() -> Option<Self> {
        let path = Self::path();
        let content = fs::read_to_string(&path).ok()?;
        toml::from_str(&content).ok()
    }

    pub fn save(&self) {
        let path = Self::path();
        fs::create_dir_all(path.parent().unwrap()).expect("failed to create config directory");
        let content = toml::to_string_pretty(self).expect("failed to serialize config");
        fs::write(&path, content).expect("failed to write config file");
    }
}

pub fn sanitize_path(input: &str) -> String {
    let trimmed = input.trim();
    let stripped = if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    let stripped = stripped.trim();
    // if the user provided with a home dir shorthand extract the actual home dir path to generate an absolute path
    if stripped.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&stripped[2..]).to_string_lossy().to_string();
        }
    }
    stripped.to_string()
}

/// just a really base check of metadata.db
pub fn is_calibre_library(path: &Path) -> bool {
    path.is_dir() && path.join("metadata.db").is_file()
}

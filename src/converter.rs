use std::path::{Path, PathBuf};
use std::process::Command;

use crate::scanner;

const EBOOK_CONVERT_TARGETS: &[&str] = &["EPUB", "PDF", "MOBI", "AZW3", "KEPUB", "DOCX", "TXT"];

/// Returns `(has_kepubify, has_ebook_convert)`.
pub fn available_backends() -> (bool, bool) {
    let has_kepubify = Command::new("kepubify")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    let has_ebook_convert = Command::new("ebook-convert")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    (has_kepubify, has_ebook_convert)
}

/// Given a book's current formats string (e.g. "EPUB, PDF") and which backends
/// are installed, return the list of (format, tool) pairs the user can convert to.
pub fn target_formats(
    existing_formats: &str,
    has_kepubify: bool,
    has_calibre_convert: bool,
) -> Vec<(String, String)> {
    let owned: Vec<String> = existing_formats
        .split(", ")
        .map(|s| s.to_string())
        .collect();
    let has_epub_file = owned.iter().any(|f| f == "EPUB");
    let mut targets: Vec<(String, String)> = Vec::new();

    if has_kepubify && has_epub_file && !owned.contains(&"KEPUB".to_string()) {
        targets.push(("KEPUB".to_string(), "kepubify".to_string()));
    }

    if has_calibre_convert {
        for &fmt in EBOOK_CONVERT_TARGETS {
            if !owned.contains(&fmt.to_string())
                && !targets.iter().any(|(f, _)| f == fmt)
            {
                targets.push((fmt.to_string(), "ebook-convert".to_string()));
            }
        }
    }

    targets
}

/// Find a source file in the book directory suitable for conversion.
/// Prefers EPUB when available, otherwise picks the first recognized format.
pub fn find_source_file(book_path: &Path) -> Option<PathBuf> {
    let entries: Vec<_> = std::fs::read_dir(book_path)
        .ok()?
        .flatten()
        .filter(|e| e.path().is_file())
        .collect();

    // Prefer EPUB source (but not .kepub.epub)
    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let lower = name.to_lowercase();
        if lower.ends_with(".epub") && !lower.ends_with(".kepub.epub") {
            return Some(entry.path());
        }
    }

    // Fallback: first file with a recognized format
    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if scanner::extract_format(&name).is_some() {
            return Some(entry.path());
        }
    }

    None
}

/// Run the conversion and return a success message or error.
pub fn convert(
    book_path: &Path,
    source_file: &Path,
    target_format: &str,
) -> Result<String, String> {
    // TODO fix: when the command is running the whole view is blocked,
    let source_name = source_file
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    let (has_kepubify, _) = available_backends();
    let use_kepubify = target_format == "KEPUB"
        && source_name.to_lowercase().ends_with(".epub")
        && has_kepubify;

    if use_kepubify {
        let output = Command::new("kepubify")
            .arg("-o")
            .arg(book_path)
            .arg(source_file)
            .output()
            .map_err(|e| format!("Failed to run kepubify: {e}"))?;

        if output.status.success() {
            Ok(format!("Converted to KEPUB successfully"))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("kepubify failed: {}", stderr.trim()))
        }
    } else {
        // Use ebook-convert
        let stem = source_file
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        // Handle .kepub.epub double extension for stem
        let stem = stem.strip_suffix(".kepub").unwrap_or(&stem);
        let ext = target_format.to_lowercase();
        let output_path = book_path.join(format!("{stem}.{ext}"));

        let output = Command::new("ebook-convert")
            .arg(source_file)
            .arg(&output_path)
            .output()
            .map_err(|e| format!("Failed to run ebook-convert: {e}"))?;

        if output.status.success() {
            Ok(format!("Converted to {target_format} successfully"))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("ebook-convert failed: {}", stderr.trim()))
        }
    }
}

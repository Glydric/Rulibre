use std::{fs, path::Path, process};

#[derive(Clone)]
pub struct Book {
    pub author: String,
    pub title: String,
    pub formats: String,
}

// files are automatically ignored
const SKIP_TOP_LEVEL_FOLDERS: &[&str] = &[".caltrash", ".calnotes", ".DS_Store", "downloaded"];

const SKIP_BOOK_FILES: &[&str] = &["metadata.opf", "cover.jpg", ".DS_Store"];

pub fn scan_library(library_path: &Path) -> Vec<Book> {
    let mut books = Vec::new();

    let Ok(author_dirs) = fs::read_dir(library_path) else {
        eprintln!("Failed to read library directory");
        process::exit(1);
    };

    for author_entry in author_dirs.flatten() {
        let author_name = author_entry.file_name().to_string_lossy().to_string();
        if SKIP_TOP_LEVEL_FOLDERS.contains(&author_name.as_str()) {
            continue;
        }
        if !author_entry.path().is_dir() {
            continue;
        }

        let Ok(title_dirs) = fs::read_dir(author_entry.path()) else {
            continue;
        };

        for title_entry in title_dirs.flatten() {
            if !title_entry.path().is_dir() {
                continue;
            }
            let raw_title = title_entry.file_name().to_string_lossy().to_string();
            let title = strip_calibre_id(&raw_title);

            let Ok(book_files) = fs::read_dir(title_entry.path()) else {
                continue;
            };

            let mut formats: Vec<String> = Vec::new();
            for file_entry in book_files.flatten() {
                let fname = file_entry.file_name().to_string_lossy().to_string();
                if SKIP_BOOK_FILES.contains(&fname.as_str()) {
                    continue;
                }
                if !file_entry.path().is_file() {
                    continue;
                }
                let path = file_entry.path();
                let Some(ext) = path.extension() else {
                    continue;
                };
                let ext_upper = ext.to_string_lossy().to_uppercase();
                if !formats.contains(&ext_upper) {
                    formats.push(ext_upper);
                }
            }

            if !formats.is_empty() {
                formats.sort();
                books.push(Book {
                    title,
                    author: author_name.clone(),
                    formats: formats.join(", "),
                });
            }
        }
    }

    books.sort_by(|a, b| {
        a.author
            .to_lowercase()
            .cmp(&b.author.to_lowercase())
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
    });

    books
}

fn strip_calibre_id(dir_name: &str) -> String {
    if let Some(idx) = dir_name.rfind(" (")
        && dir_name.ends_with(')')
    {
        let between = &dir_name[idx + 2..dir_name.len() - 1];
        if between.chars().all(|c| c.is_ascii_digit()) {
            return dir_name[..idx].to_string();
        }
    }
    dir_name.to_string()
}

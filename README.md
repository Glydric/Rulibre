# rulibre

A terminal UI for browsing your Calibre library. Displays books in a table with Author, Title, and Format columns.

## Usage

```
cargo run
```

On first run, you'll be prompted for your Calibre library path. The path is saved to `~/.config/rulibre/config.toml`.

### Controls

| Key       | Action    |
|-----------|-----------|
| `j` / `↓` | Next row  |
| `k` / `↑` | Prev row  |
| `q` / `Esc` | Quit    |

## What gets scanned

The scanner walks `{library}/{author}/{title (id)}/` directories and picks up book files by extension (EPUB, CBZ, PDF, etc.).

Excluded from scanning:
- `metadata.db`, `metadata_db_prefs_backup.json` (Calibre internal)
- `.caltrash/`, `.calnotes/`, `.DS_Store`, `downloaded/` (Calibre internal)
- `metadata.opf`, `cover.jpg` inside book folders

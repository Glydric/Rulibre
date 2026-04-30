# Rulibre

I got tired of Calibre's old, complex interface and its heavy storage footprint. All I really needed was a way to browse my library and transfer books to my e-reader, so I built a lightweight, drop-in replacement that works with your existing Calibre library structure. I also added things I found lacking or too complicated to use in Calibre, like instant book search.

![Rulibre screenshot](assets/screenshot.png)

## Install

The easiest way to get Rulibre is from [crates.io](https://crates.io/crates/rulibre):

```
cargo install rulibre
rulibre
```

This installs the **TUI** (terminal) version — a single, dependency-free binary. The GUI version is source-only (see [Build from source](#build-from-source)).

On first run, you'll be prompted for your Calibre library path. The path is saved to your system default config path.

### Uninstall

```
cargo uninstall rulibre
```

## Features

- Browse all books in your Calibre library sorted by author and title
- Search/filter with `/` across author, title, and format
- Detail panel showing metadata parsed from `metadata.opf` (title, author, publisher, date, language, series, tags, description, identifiers)
- Convert books between formats (`c` key) using [kepubify](https://pgaskin.net/kepubify/) or Calibre's `ebook-convert`
- Send books to a connected e-reader (`t` key) — auto-detects Kobo and Kindle mounts
- Mouse support: click to select books, scroll both panels
- Keyboard focus switching between table and detail panels

## What gets scanned

The scanner walks `{library}/{author}/{title (id)}/` directories and picks up book files by extension (EPUB, KEPUB, CBZ, PDF, etc.).

Excluded from scanning:
- `.caltrash/`, `.calnotes/`, `.DS_Store`, `downloaded/` (Calibre internal)
- `metadata.opf`, `cover.jpg` inside book folders

## Book conversion

Press `c` on a selected book to convert it to another format. Conversion options are shown based on which tools are installed:

- **kepubify** — used for EPUB → KEPUB conversion. Install from [pgaskin.net/kepubify](https://pgaskin.net/kepubify/)
- **ebook-convert** (Calibre) — used for all other conversions (EPUB, PDF, MOBI, AZW3, KEPUB, DOCX, TXT). Included with [Calibre](https://calibre-ebook.com/)

Only formats you don't already have are offered. If neither tool is installed, the convert dialog will let you know.

## Build from source

Rulibre ships two frontends from a single workspace:

- **TUI** — the ratatui terminal interface, published on crates.io
- **GUI** — Tauri 2 + Leptos desktop window (source-only)

### TUI

```
cargo install --path crates/rulibre
```

Or run directly from a checkout:

```
cargo run -p rulibre
```

### GUI

Prerequisites:

```
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
cargo install --locked tauri-cli --version "^2"
```

Development build:

```
cd crates/rulibre-gui
cargo tauri dev
```

Release build:

```
cd crates/rulibre-gui
cargo tauri build
```

## Extra tools

A diagnostic binary is included for checking unrecognized metadata tags:

```
cargo run --bin scan_metadata -- /path/to/calibre/library
```

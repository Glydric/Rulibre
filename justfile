tui-run:
    cargo run

tui-build:
    cargo build

install-gui-prerequirements:
    rustup target add wasm32-unknown-unknown
    cargo install --locked trunk tauri-cli

[working-directory('crates/rulibre-gui')]
gui-dev:
    cargo tauri dev

[working-directory("crates/rulibre-gui")]
gui-build:
    cargo tauri build

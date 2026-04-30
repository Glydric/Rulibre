use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlElement, KeyboardEvent};

use crate::state::{AppState, Focus, Mode};
use crate::ipc;
use crate::types::Metadata;

/// Returns true if the key event originated from a text input — in which case
/// global shortcuts should be skipped (e.g. typing into the search bar).
fn target_is_input(ev: &KeyboardEvent) -> bool {
    ev.target()
        .and_then(|t| t.dyn_into::<HtmlElement>().ok())
        .map(|el| {
            let tag = el.tag_name().to_lowercase();
            tag == "input" || tag == "textarea"
        })
        .unwrap_or(false)
}

pub fn handle_global_keydown(state: AppState, ev: KeyboardEvent) {
    let mode = state.mode.get_untracked();
    if matches!(mode, Mode::Search) {
        // Search input owns its own onkeydown handler.
        return;
    }
    if target_is_input(&ev) && !matches!(mode, Mode::Setup) {
        return;
    }
    let key = ev.key();
    match mode {
        Mode::Normal => normal(state, &key, &ev),
        Mode::Detail => detail(state, &key, &ev),
        Mode::Convert => convert(state, &key, &ev),
        Mode::Search => unreachable!(),
        Mode::Setup => setup_global(state, &key, &ev),
    }
}

fn normal(state: AppState, key: &str, ev: &KeyboardEvent) {
    match key {
        "q" | "Escape" => {
            // Closing the window is delegated to Tauri's default close shortcut;
            // no-op here so unsaved state isn't lost.
        }
        "ArrowDown" | "s" => {
            state.next();
            ev.prevent_default();
        }
        "ArrowUp" | "w" => {
            state.previous();
            ev.prevent_default();
        }
        "/" => {
            state.mode.set(Mode::Search);
            ev.prevent_default();
        }
        "Enter" => open_detail(state),
        "c" => enter_convert(state),
        "t" => send_to_device(state),
        _ => {}
    }
}

fn detail(state: AppState, key: &str, ev: &KeyboardEvent) {
    match key {
        "Escape" | "q" => close_detail(state),
        "ArrowLeft" | "a" => state.focus.set(Focus::Table),
        "ArrowRight" | "d" => state.focus.set(Focus::Detail),
        "ArrowDown" | "s" => match state.focus.get_untracked() {
            Focus::Table => {
                state.next();
                refresh_detail(state);
                ev.prevent_default();
            }
            Focus::Detail => {} // browser handles scroll
        },
        "ArrowUp" | "w" => match state.focus.get_untracked() {
            Focus::Table => {
                state.previous();
                refresh_detail(state);
                ev.prevent_default();
            }
            Focus::Detail => {}
        },
        "c" => enter_convert(state),
        "t" => send_to_device(state),
        "Enter" => match state.focus.get_untracked() {
            Focus::Table => open_detail(state),
            Focus::Detail => close_detail(state),
        },
        _ => {}
    }
}

fn convert(state: AppState, key: &str, ev: &KeyboardEvent) {
    match key {
        "Escape" | "q" => {
            state.mode.set(Mode::Normal);
            state.convert_targets.set(Vec::new());
            state.convert_message.set(None);
        }
        "ArrowDown" | "s" => {
            let targets = state.convert_targets.get();
            if !targets.is_empty() {
                let cur = state.convert_selected.get();
                state.convert_selected.set((cur + 1) % targets.len());
            }
            ev.prevent_default();
        }
        "ArrowUp" | "w" => {
            let targets = state.convert_targets.get();
            if !targets.is_empty() {
                let cur = state.convert_selected.get();
                let next = if cur == 0 { targets.len() - 1 } else { cur - 1 };
                state.convert_selected.set(next);
            }
            ev.prevent_default();
        }
        "Enter" => run_convert(state),
        _ => {}
    }
}

fn setup_global(_state: AppState, _key: &str, _ev: &KeyboardEvent) {
    // Setup mode is fully handled inside the input element.
}

pub fn open_detail(state: AppState) {
    let Some(book) = state.selected_book() else {
        return;
    };
    state.mode.set(Mode::Detail);
    state.focus.set(Focus::Table);
    spawn_local(async move {
        let meta = ipc::parse_metadata(book.path).await;
        state.detail.set(meta.or(Some(Metadata::default())));
    });
}

pub fn refresh_detail(state: AppState) {
    let Some(book) = state.selected_book() else {
        return;
    };
    spawn_local(async move {
        let meta = ipc::parse_metadata(book.path).await;
        state.detail.set(meta.or(Some(Metadata::default())));
    });
}

pub fn close_detail(state: AppState) {
    state.mode.set(Mode::Normal);
    state.focus.set(Focus::Table);
    state.detail.set(None);
}

pub fn enter_convert(state: AppState) {
    let Some(book) = state.selected_book() else {
        return;
    };
    spawn_local(async move {
        let (k, c) = ipc::available_backends().await;
        state.backends.set((k, c));
        if !k && !c {
            state.convert_targets.set(Vec::new());
            state.convert_message.set(Some(Err(
                "No conversion tools found (install kepubify or calibre's ebook-convert)".into(),
            )));
            state.mode.set(Mode::Convert);
            return;
        }
        let targets = ipc::target_formats(book.formats.clone(), k, c).await;
        if targets.is_empty() {
            state.convert_targets.set(Vec::new());
            state
                .convert_message
                .set(Some(Err("No formats to convert to".into())));
            state.mode.set(Mode::Convert);
            return;
        }
        state.convert_targets.set(targets);
        state.convert_selected.set(0);
        state.convert_message.set(None);
        state.mode.set(Mode::Convert);
    });
}

pub fn run_convert(state: AppState) {
    let targets = state.convert_targets.get();
    if targets.is_empty() {
        return;
    }
    let Some(book) = state.selected_book() else {
        return;
    };
    let (target, _tool) = targets[state.convert_selected.get()].clone();
    let book_path = book.path.clone();
    spawn_local(async move {
        match ipc::convert_book(book_path.clone(), target).await {
            Ok(ok) => {
                state.all_books.update(|books| {
                    for b in books.iter_mut() {
                        if b.path == book_path {
                            b.formats = ok.new_formats.clone();
                        }
                    }
                });
                state.convert_message.set(Some(Ok(ok.message.clone())));
                state.convert_targets.set(Vec::new());
                state.mode.set(Mode::Normal);
                state.notify(Ok(ok.message));
            }
            Err(err) => {
                state.convert_message.set(Some(Err(err)));
            }
        }
    });
}

pub fn send_to_device(state: AppState) {
    let Some(book) = state.selected_book() else {
        return;
    };
    spawn_local(async move {
        let result = ipc::send_to_device(book.path, book.author).await;
        state.notify(result);
    });
}

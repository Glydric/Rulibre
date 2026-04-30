use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::state::{AppState, Mode};
use crate::ipc;

#[component]
pub fn Setup() -> impl IntoView {
    let state = expect_context::<AppState>();

    let on_input = move |ev| {
        let value = event_target_value(&ev);
        state.setup_input.set(value);
        state.setup_error.set(String::new());
    };

    let submit = move || {
        let path = state.setup_input.get();
        spawn_local(async move {
            match ipc::validate_library(path.clone()).await {
                Ok(sanitized) => {
                    if let Err(err) = ipc::save_config(sanitized.clone()).await {
                        state.setup_error.set(err);
                        return;
                    }
                    let books = ipc::scan_library(sanitized).await;
                    let has_books = !books.is_empty();
                    state.all_books.set(books);
                    state.selected_idx.set(if has_books { Some(0) } else { None });
                    state.mode.set(Mode::Normal);
                }
                Err(err) => state.setup_error.set(err),
            }
        });
    };

    let on_click = move |_: leptos::ev::MouseEvent| submit();
    let on_key = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            submit();
        }
    };

    view! {
        <div class="modal-scrim">
            <div class="modal" role="dialog">
                <h2>"Enter Calibre library path"</h2>
                <input
                    class="modal-input"
                    type="text"
                    autofocus="true"
                    placeholder="~/Calibre Library"
                    prop:value=move || state.setup_input.get()
                    on:input=on_input
                    on:keydown=on_key
                />
                <div class="modal-error">{move || state.setup_error.get()}</div>
                <div class="modal-hint">
                    <span><kbd>"Enter"</kbd>" confirm"</span>
                </div>
                <div class="modal-actions">
                    <button on:click=on_click>"Open library"</button>
                </div>
            </div>
        </div>
    }
}

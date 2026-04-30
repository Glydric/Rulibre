use leptos::prelude::*;

use crate::state::{AppState, Mode};

#[component]
pub fn Convert() -> impl IntoView {
    let state = expect_context::<AppState>();

    let visible = move || matches!(state.mode.get(), Mode::Convert);

    let title = move || {
        let book = state.selected_book();
        let title = book.map_or_else(|| "Unknown".to_string(), |b| b.title);
        format!("Convert: {title}")
    };

    view! {
        <Show when=visible fallback=|| view! { <></> }>
            <div class="modal-scrim">
                <div class="modal convert-modal" role="dialog">
                    <h2>{title}</h2>
                    <ConvertList />
                    <ConvertMessage />
                    <ConvertHints />
                </div>
            </div>
        </Show>
    }
}

#[component]
fn ConvertList() -> impl IntoView {
    let state = expect_context::<AppState>();

    view! {
        <ul class="convert-list">
            <For
                each={move || state.convert_targets.get().into_iter().enumerate().collect::<Vec<_>>()}
                key={|item| item.0}
                children={move |item| convert_row(state, item)}
            />
        </ul>
    }
}

fn convert_row(state: AppState, item: (usize, (String, String))) -> impl IntoView {
    let (i, (fmt, tool)) = item;
    let row_class = move || {
        if state.convert_selected.get() == i { "convert-row selected" } else { "convert-row" }
    };
    let marker = move || if state.convert_selected.get() == i { "▶" } else { " " };
    let on_click = move |_| state.convert_selected.set(i);
    view! {
        <li class=row_class on:click=on_click>
            <span class="convert-marker">{marker}</span>
            <span class="convert-fmt">{fmt}</span>
            <span class="convert-tool">{format!("[{tool}]")}</span>
        </li>
    }
}

#[component]
fn ConvertMessage() -> impl IntoView {
    let state = expect_context::<AppState>();

    let body = move || match state.convert_message.get() {
        Some(Ok(msg)) => view! { <p class="convert-msg ok">{msg}</p> }.into_any(),
        Some(Err(msg)) => view! { <p class="convert-msg err">{msg}</p> }.into_any(),
        None => view! { <></> }.into_any(),
    };

    view! { {body} }
}

#[component]
fn ConvertHints() -> impl IntoView {
    let state = expect_context::<AppState>();

    let body = move || {
        if state.convert_targets.get().is_empty() {
            view! {
                <div class="modal-hint">
                    <span><kbd>"Esc"</kbd>" close"</span>
                </div>
            }
            .into_any()
        } else {
            view! {
                <div class="modal-hint">
                    <span><kbd>"Enter"</kbd>" convert"</span>
                    <span><kbd>"Esc"</kbd>" cancel"</span>
                </div>
            }
            .into_any()
        }
    };

    view! { {body} }
}

use leptos::prelude::*;
use leptos::ev::KeyboardEvent;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use crate::state::{AppState, Mode};

#[component]
pub fn Search() -> impl IntoView {
    let state = expect_context::<AppState>();

    let visible = move || matches!(state.mode.get(), Mode::Search);

    let on_input = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        state.search_query.set(value);
        state.after_apply_filter();
    };

    let on_key = move |ev: KeyboardEvent| {
        ev.stop_propagation();
        match ev.key().as_str() {
            "Escape" => {
                state.search_query.set(String::new());
                state.after_apply_filter();
                state.mode.set(Mode::Normal);
            }
            "Enter" => {
                state.mode.set(Mode::Normal);
            }
            _ => {}
        }
    };

    let input_ref = NodeRef::<leptos::html::Input>::new();

    Effect::new(move |_| {
        if visible()
            && let Some(el) = input_ref.get()
        {
            let _ = el.unchecked_ref::<HtmlInputElement>().focus();
        }
    });

    view! {
        <Show when=visible fallback=|| view! { <></> }>
            <div class="search-bar">
                <span class="search-prefix"><kbd>"/"</kbd></span>
                <input
                    class="search-input"
                    type="text"
                    node_ref=input_ref
                    prop:value=move || state.search_query.get()
                    on:input=on_input
                    on:keydown=on_key
                />
            </div>
        </Show>
    }
}

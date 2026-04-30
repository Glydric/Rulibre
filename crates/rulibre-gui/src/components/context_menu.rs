use leptos::prelude::*;

use crate::keys;
use crate::state::AppState;

#[component]
pub fn ContextMenu() -> impl IntoView {
    let state = expect_context::<AppState>();

    let visible = move || state.context_menu.get().is_some();
    let menu_style = move || {
        state.context_menu.get()
            .map(|m| format!("left:{}px;top:{}px", m.x, m.y))
            .unwrap_or_default()
    };
    let close = move |_| state.context_menu.set(None);
    let on_open = move |_| dispatch(state, keys::open_detail);
    let on_convert = move |_| dispatch(state, keys::enter_convert);
    let on_send = move |_| dispatch(state, keys::send_to_device);
    let has_device = move || state.device.get().is_some();

    view! {
        <Show when=visible fallback=|| ()>
            <div class="context-overlay" on:click=close></div>
            <div class="context-menu" style=menu_style>
                <button class="context-item" on:click=on_open>"Open Details"</button>
                <button class="context-item" on:click=on_convert>"Convert…"</button>
                <Show when=has_device fallback=|| ()>
                    <button class="context-item" on:click=on_send>"Send to Device"</button>
                </Show>
            </div>
        </Show>
    }
}

fn dispatch(state: AppState, action: impl FnOnce(AppState)) {
    if let Some(m) = state.context_menu.get_untracked() {
        state.context_menu.set(None);
        state.selected_idx.set(Some(m.book_idx));
        action(state);
    }
}

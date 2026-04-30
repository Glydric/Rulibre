use leptos::prelude::*;

use crate::components::search::Search;
use crate::keys;
use crate::state::{AppState, Focus, Mode};

#[component]
pub fn StatusBar() -> impl IntoView {
    let state = expect_context::<AppState>();

    let body = move || match state.mode.get() {
        Mode::Setup => view! { <></> }.into_any(),
        Mode::Search => view! { <Search /> }.into_any(),
        Mode::Detail => detail_hints(state).into_any(),
        Mode::Convert => convert_hints(state).into_any(),
        Mode::Normal => normal_hints(state).into_any(),
    };

    view! {
        <footer class="status-bar">{body}</footer>
    }
}

fn hint(k: &'static str, text: &'static str, on_click: impl Fn() + 'static) -> AnyView {
    view! {
        <button class="hint-btn" on:click=move |_| on_click()>
            <kbd>{k}</kbd>
            <span class="hint-label">{text}</span>
        </button>
    }
    .into_any()
}

fn hint_pair(k1: &'static str, k2: &'static str, text: &'static str, on_click: impl Fn() + 'static) -> AnyView {
    view! {
        <button class="hint-btn" on:click=move |_| on_click()>
            <kbd>{k1}</kbd>"/"<kbd>{k2}</kbd>
            <span class="hint-label">{text}</span>
        </button>
    }
    .into_any()
}

fn action_btn(text: &'static str, on_click: impl Fn() + 'static) -> AnyView {
    view! {
        <button class="hint-btn hint-action" on:click=move |_| on_click()>
            {text}
        </button>
    }
    .into_any()
}

fn normal_hints(state: AppState) -> impl IntoView {
    let has_device = move || state.device.get().is_some();
    view! {
        {hint_pair("w", "↑", " up", move || state.previous())}
        {hint_pair("s", "↓", " down", move || state.next())}
        {hint("Enter", " detail", move || keys::open_detail(state))}
        {hint("c", " convert", move || keys::enter_convert(state))}
        <Show when=has_device fallback=|| ()>
            {hint("t", " send", move || keys::send_to_device(state))}
        </Show>
        {hint("/", " search", move || state.mode.set(Mode::Search))}
    }
}

fn detail_hints(state: AppState) -> impl IntoView {
    let has_device = move || state.device.get().is_some();
    view! {
        {hint_pair("←", "→", " focus", move || {
            let next = if matches!(state.focus.get_untracked(), Focus::Table) {
                Focus::Detail
            } else {
                Focus::Table
            };
            state.focus.set(next);
        })}
        {hint_pair("w", "↑", " up", move || { state.previous(); keys::refresh_detail(state); })}
        {hint_pair("s", "↓", " down", move || { state.next(); keys::refresh_detail(state); })}
        {hint("c", " convert", move || keys::enter_convert(state))}
        <Show when=has_device fallback=|| ()>
            {hint("t", " send", move || keys::send_to_device(state))}
        </Show>
        {action_btn("close", move || keys::close_detail(state))}
    }
}

fn convert_hints(state: AppState) -> impl IntoView {
    view! {
        {hint_pair("w", "↑", " up", move || {
            let targets = state.convert_targets.get_untracked();
            if !targets.is_empty() {
                let cur = state.convert_selected.get_untracked();
                let prev = if cur == 0 { targets.len() - 1 } else { cur - 1 };
                state.convert_selected.set(prev);
            }
        })}
        {hint_pair("s", "↓", " down", move || {
            let targets = state.convert_targets.get_untracked();
            if !targets.is_empty() {
                let cur = state.convert_selected.get_untracked();
                state.convert_selected.set((cur + 1) % targets.len());
            }
        })}
        {hint("Enter", " convert", move || keys::run_convert(state))}
        {action_btn("cancel", move || {
            state.mode.set(Mode::Normal);
            state.convert_targets.set(Vec::new());
            state.convert_message.set(None);
        })}
    }
}

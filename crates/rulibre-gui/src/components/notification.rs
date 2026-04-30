use leptos::prelude::*;

use crate::state::AppState;

#[component]
pub fn Notification() -> impl IntoView {
    let state = expect_context::<AppState>();

    let body = move || match state.notification.get() {
        Some(Ok(msg)) => view! { <div class="toast toast-ok">{msg}</div> }.into_any(),
        Some(Err(msg)) => view! { <div class="toast toast-err">{msg}</div> }.into_any(),
        None => view! { <></> }.into_any(),
    };

    view! { {body} }
}

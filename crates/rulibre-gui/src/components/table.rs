use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::MouseEvent;

use crate::state::{AppState, ContextMenuData, Focus, Mode};
use crate::ipc;
use crate::types::{Book, Metadata};

#[component]
pub fn BookTable() -> impl IntoView {
    let state = expect_context::<AppState>();

    Effect::new(move |_| {
        let total = state.all_books.get().len();
        let shown = state.filtered.get().len();
        let q = state.search_query.get();
        let title = if q.is_empty() {
            format!("Rulibre — {} books", total)
        } else {
            format!("Rulibre — {} / {} books", shown, total)
        };
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            doc.set_title(&title);
        }
    });

    let panel_class = move || {
        if matches!(state.mode.get(), Mode::Detail) && matches!(state.focus.get(), Focus::Table) {
            "panel focused"
        } else {
            "panel"
        }
    };

    view! {
        <section class=panel_class>
            <div class="panel-body">
                <table class="book-table">
                    <thead>
                        <tr>
                            <th style="width:30%">"Author"</th>
                            <th style="width:50%">"Title"</th>
                            <th style="width:20%">"Format"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each={move || state.filtered.get().into_iter().enumerate().collect::<Vec<_>>()}
                            key=|(_, b)| format!("{}:{}", b.path, b.formats)
                            children=move |(idx, book): (usize, Book)| book_row(state, idx, book)
                        />
                    </tbody>
                </table>
            </div>
        </section>
    }
}

fn book_row(state: AppState, idx: usize, book: Book) -> impl IntoView {
    let row_class = move || {
        if state.selected_idx.get() == Some(idx) { "selected" } else { "" }
    };
    let on_click = move |_| open_row(state, idx);
    let on_contextmenu = move |ev: MouseEvent| {
        ev.prevent_default();
        state.context_menu.set(Some(ContextMenuData {
            x: ev.client_x(),
            y: ev.client_y(),
            book_idx: idx,
        }));
    };
    view! {
        <tr class=row_class on:click=on_click on:contextmenu=on_contextmenu>
            <td>{book.author}</td>
            <td>{book.title}</td>
            <td class="col-format">{book.formats}</td>
        </tr>
    }
}

pub fn open_row(state: AppState, idx: usize) {
    state.selected_idx.set(Some(idx));
    state.focus.set(Focus::Table);
    let book = state.filtered.get().get(idx).cloned();
    if let Some(book) = book {
        state.mode.set(Mode::Detail);
        spawn_local(async move {
            let meta = ipc::parse_metadata(book.path).await;
            state.detail.set(meta.or(Some(Metadata::default())));
        });
    }
}

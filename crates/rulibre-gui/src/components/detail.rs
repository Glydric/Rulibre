use leptos::prelude::*;

use crate::state::{AppState, Focus, Mode};
use crate::types::Metadata;

#[component]
pub fn Detail() -> impl IntoView {
    let state = expect_context::<AppState>();

    let panel_class = move || {
        if matches!(state.focus.get(), Focus::Detail) {
            "panel detail-panel focused"
        } else {
            "panel detail-panel"
        }
    };

    let visible = move || matches!(state.mode.get(), Mode::Detail);

    let body = move || {
        let meta = state.detail.get();
        let formats = state.selected_book().map(|b| b.formats);
        match meta {
            None => view! { <p class="detail-empty">"No metadata found."</p> }.into_any(),
            Some(meta) => {
                view! { <div class="detail-content">{sections(meta, formats)}</div> }.into_any()
            }
        }
    };

    view! {
        <Show when=visible fallback=|| view! { <></> }>
            <section class=panel_class>
                <header class="panel-title">"Detail"</header>
                <div class="panel-body detail-body">{body}</div>
            </section>
        </Show>
    }
}

fn sections(meta: Metadata, formats: Option<String>) -> AnyView {
    view! {
        {book_info(&meta, formats)}
        {publishing(&meta)}
        {tags(&meta)}
        {description(&meta)}
        {unrecognized(&meta)}
    }
    .into_any()
}

fn field(label: String, value: String) -> AnyView {
    view! {
        <div class="detail-row">
            <span class="detail-key">{label}</span>
            <span class="detail-val">{value}</span>
        </div>
    }
    .into_any()
}

fn book_info(meta: &Metadata, formats: Option<String>) -> AnyView {
    let authors = (!meta.authors.is_empty()).then(|| meta.authors.join(", "));
    let series = (!meta.series.is_empty()).then(|| {
        if meta.series_index.is_empty() {
            meta.series.clone()
        } else {
            format!("{} #{}", meta.series, meta.series_index)
        }
    });

    view! {
        <section class="detail-section">
            <h3>"Book Info"</h3>
            {field("Title".into(), meta.title.clone())}
            {authors.map(|a| field("Author".into(), a))}
            {formats.map(|f| field("Formats".into(), f))}
            {series.map(|s| field("Series".into(), s))}
        </section>
    }
    .into_any()
}

fn publishing(meta: &Metadata) -> AnyView {
    let any = !meta.publisher.is_empty()
        || !meta.date.is_empty()
        || !meta.language.is_empty()
        || !meta.identifiers.is_empty()
        || !meta.rating.is_empty();
    if !any {
        return ().into_any();
    }

    let identifiers: Vec<_> = meta
        .identifiers
        .iter()
        .map(|(scheme, value)| field(format!("{scheme}:"), value.clone()))
        .collect();

    view! {
        <section class="detail-section">
            <h3>"Publishing"</h3>
            {(!meta.publisher.is_empty()).then(|| field("Publisher".into(), meta.publisher.clone()))}
            {(!meta.date.is_empty()).then(|| field("Date".into(), meta.date.clone()))}
            {(!meta.language.is_empty()).then(|| field("Language".into(), meta.language.clone()))}
            {identifiers}
            {(!meta.rating.is_empty()).then(|| field("Rating".into(), meta.rating.clone()))}
        </section>
    }
    .into_any()
}

fn tags(meta: &Metadata) -> AnyView {
    if meta.subjects.is_empty() {
        return ().into_any();
    }
    let joined = meta.subjects.join(", ");
    view! {
        <section class="detail-section">
            <h3>"Tags"</h3>
            <p class="detail-text">{joined}</p>
        </section>
    }
    .into_any()
}

fn description(meta: &Metadata) -> AnyView {
    if meta.description.is_empty() {
        return ().into_any();
    }
    let text = meta.description.clone();
    view! {
        <section class="detail-section">
            <h3>"Description"</h3>
            <p class="detail-text detail-description">{text}</p>
        </section>
    }
    .into_any()
}

fn unrecognized(meta: &Metadata) -> AnyView {
    if !cfg!(debug_assertions) || meta.unrecognized.is_empty() {
        return ().into_any();
    }
    let items: Vec<_> = meta
        .unrecognized
        .iter()
        .map(|tag| view! { <li>{tag.clone()}</li> })
        .collect();
    view! {
        <section class="detail-section">
            <h3>"Unknown Metadata"</h3>
            <ul class="detail-unknown">{items}</ul>
        </section>
    }
    .into_any()
}

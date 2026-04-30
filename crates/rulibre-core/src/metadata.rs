use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub authors: Vec<String>,
    pub publisher: String,
    pub date: String,
    pub language: String,
    pub subjects: Vec<String>,
    pub description: String,
    pub identifiers: Vec<(String, String)>,
    pub rating: String,
    pub series: String,
    pub series_index: String,
    pub unrecognized: Vec<String>,
}

const DC_NS: &str = "http://purl.org/dc/elements/1.1/";
const OPF_NS: &str = "http://www.idpf.org/2007/opf";

pub fn parse_opf(book_path: &Path) -> Option<Metadata> {
    let opf_path = book_path.join("metadata.opf");
    let xml = fs::read_to_string(&opf_path).ok()?;
    let doc = roxmltree::Document::parse(&xml).ok()?;

    let metadata_node = doc.descendants().find(|n| n.has_tag_name("metadata"))?;

    let mut title = String::new();
    let mut authors = Vec::new();
    let mut publisher = String::new();
    let mut date = String::new();
    let mut language = String::new();
    let mut subjects = Vec::new();
    let mut description = String::new();
    let mut identifiers = Vec::new();
    let mut rating = String::new();
    let mut series = String::new();
    let mut series_index = String::new();
    let mut unrecognized = Vec::new();

    for child in metadata_node.children().filter(|n| n.is_element()) {
        let ns = child.tag_name().namespace().unwrap_or_default();
        let local = child.tag_name().name();

        match (ns, local) {
            (DC_NS, "title") => {
                title = child.text().unwrap_or_default().to_string();
            }
            (DC_NS, "creator") => {
                if let Some(text) = child.text() {
                    authors.push(text.to_string());
                }
            }
            (DC_NS, "publisher") => {
                publisher = child.text().unwrap_or_default().to_string();
            }
            (DC_NS, "date") => {
                date = child.text().unwrap_or_default().to_string();
                if let Some(idx) = date.find('T') {
                    date.truncate(idx);
                }
            }
            (DC_NS, "language") => {
                language = child.text().unwrap_or_default().to_string();
            }
            (DC_NS, "subject") => {
                if let Some(text) = child.text() {
                    subjects.push(text.to_string());
                }
            }
            (DC_NS, "description") => {
                description = strip_html(child.text().unwrap_or_default());
            }
            (DC_NS, "identifier") => {
                let scheme = child
                    .attribute((OPF_NS, "scheme"))
                    .unwrap_or("ID")
                    .to_string();
                if scheme != "calibre" && scheme != "uuid" {
                    if let Some(text) = child.text() {
                        identifiers.push((scheme, text.to_string()));
                    }
                }
            }
            (_, "meta") => {
                let name = child.attribute("name").unwrap_or_default();
                let content = child.attribute("content").unwrap_or_default();
                match name {
                    "calibre:rating" => rating = content.to_string(),
                    "calibre:series" => series = content.to_string(),
                    "calibre:series_index" => series_index = content.to_string(),
                    other if !other.is_empty() => {
                        unrecognized.push(format!("meta[{}] = {}", other, content));
                    }
                    _ => {}
                }
            }
            (ns, tag) => {
                let label = if ns.is_empty() {
                    tag.to_string()
                } else {
                    format!("{{{ns}}}{tag}")
                };
                unrecognized.push(label);
            }
        }
    }

    Some(Metadata {
        title,
        authors,
        publisher,
        date,
        language,
        subjects,
        description,
        identifiers,
        rating,
        series,
        series_index,
        unrecognized,
    })
}

fn strip_html(input: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    // Decode common HTML entities
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

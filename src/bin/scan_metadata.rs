use std::{collections::BTreeSet, env, path::Path, process};

use rulibre::{metadata, scanner};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: scan_metadata <calibre-library-path>");
        process::exit(1);
    }

    let library_path = Path::new(&args[1]);
    if !library_path.is_dir() {
        eprintln!("Path does not exist: {}", args[1]);
        process::exit(1);
    }

    let books = scanner::scan_library(library_path);
    println!("Found {} books\n", books.len());

    let mut all_unrecognized = BTreeSet::new();
    let mut books_with_issues = 0;

    for book in &books {
        let Some(meta) = metadata::parse_opf(&book.path) else {
            println!(
                "[MISSING] {} — {} (no metadata.opf)",
                book.author, book.title
            );
            books_with_issues += 1;
            continue;
        };

        if !meta.unrecognized.is_empty() {
            println!("[UNRECOGNIZED] {} — {}:", book.author, book.title);
            for tag in &meta.unrecognized {
                println!("  - {tag}");
                all_unrecognized.insert(tag.clone());
            }
            books_with_issues += 1;
        }
    }

    println!("\n--- Summary ---");
    println!("Total books scanned: {}", books.len());
    println!("Books with issues:   {books_with_issues}");

    if all_unrecognized.is_empty() {
        println!("No unrecognized metadata tags found.");
    } else {
        println!(
            "\nAll unique unrecognized tags ({}):",
            all_unrecognized.len()
        );
        for tag in &all_unrecognized {
            println!("  - {tag}");
        }
    }
}

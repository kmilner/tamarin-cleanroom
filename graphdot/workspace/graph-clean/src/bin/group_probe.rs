//! Exploration: build a record GROUP as a single Doc from its cell flats, using
//! various combinators, render at a width, and print raw output (newlines shown
//! as \n, leading spaces visible). Used to reverse-engineer how a group's cells
//! share the row.
use graph_clean::doclayout::cell_doc;
use graph_clean::pretty::*;

fn show(label: &str, d: &Doc, w: isize, ribbon: f64) {
    let s = render_page(w, ribbon, d);
    println!("--- {label} @w={w} rib={ribbon}");
    for line in s.split('\n') {
        println!("   |{}", line);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // cells as separate args; width via env W (default 87)
    let cells: Vec<String> = args[1..].to_vec();
    let w: isize = std::env::var("W").ok().and_then(|s| s.parse().ok()).unwrap_or(87);
    let rib: f64 = std::env::var("RIB").ok().and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let docs: Vec<Doc> = cells.iter().map(|c| cell_doc(c)).collect();

    show("sep", &sep(docs.clone()), w, rib);
    show("cat", &cat(docs.clone()), w, rib);
    show("fsep", &fsep(docs.clone()), w, rib);
    show("fcat", &fcat(docs.clone()), w, rib);
    show("hsep", &hsep(docs.clone()), w, rib);
    show("vcat", &vcat(docs.clone()), w, rib);
    // sep/fsep with zero-width '|' separators interleaved
    let mut sep_pipe: Vec<Doc> = Vec::new();
    for (i, d) in docs.iter().enumerate() {
        if i > 0 {
            sep_pipe.push(sized_text(0, "|"));
        }
        sep_pipe.push(d.clone());
    }
    show("sep|0w", &sep(sep_pipe.clone()), w, rib);
    show("fsep|0w", &fsep(sep_pipe), w, rib);
}

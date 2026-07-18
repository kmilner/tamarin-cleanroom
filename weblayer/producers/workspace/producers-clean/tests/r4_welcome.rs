//! R4 acceptance — the welcome / index page and housekeeping bodies.
//!
//! Every target is a LIVE capture from the sanctioned oracle
//! (workspace/r4_live/, QUERIES.log [L19]–[L21]) — the crawl corpus holds no
//! non-`/thy` bodies. Each captured index page is strictly inverted (flash
//! paragraph, version text, table rows sliced out; every other byte belongs
//! to the fixed frame), rebuilt through `render_welcome`, and compared
//! byte-for-byte. The plain-text housekeeping bodies and the
//! Invalid-Arguments page are compared against their captures directly.

use std::fs;
use std::path::PathBuf;

use producers_clean::model::{Banner, TheoryRow, Welcome};
use producers_clean::render_welcome;
use producers_clean::welcome::{
    render_invalid_args, CANCEL_ACK_BODY, FILE_NOT_FOUND_BODY, ROBOTS_BODY,
};

mod common;
use common::unescape_entities;

fn r4(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../r4_live").join(rel);
    fs::read_to_string(p).unwrap_or_else(|e| panic!("r4_live/{rel}: {e}"))
}

/// Slice the three producer slots out of a captured index page.
fn invert_index(page: &str) -> Welcome {
    let body = page.find("<body>").expect("body opens") + "<body>".len();
    let banner = match page[body..].strip_prefix("<p class=\"message\">") {
        None => Banner::None,
        Some(rest) => {
            let (msg, _) = rest.split_once("</p>").expect("flash closes");
            match msg {
                "Loaded new theory!" => Banner::Loaded,
                "Post request failed." => Banner::Failed,
                other => Banner::Custom(unescape_entities(other)),
            }
        }
    };
    let vslot = "Tamarin</span></a> ";
    let vi = page.find(vslot).expect("version slot") + vslot.len();
    let version = &page[vi..vi + page[vi..].find("</div>").expect("version ends")];
    let ri = page.find("</thead>").expect("table head") + "</thead>".len();
    let rows_raw = &page[ri..ri + page[ri..].find("</table>").expect("table closes")];
    let mut rows = Vec::new();
    let mut rest = rows_raw;
    while !rest.is_empty() {
        let r = rest
            .strip_prefix("<tr><td><a href=\"/thy/trace/")
            .unwrap_or_else(|| panic!("row shape: {rest:?}"));
        let (idx, r) = r.split_once("/overview/help\">").expect("row href");
        let (name, r) = r.split_once("</a></td><td>").expect("row name cell");
        let (time, r) = r.split_once("</td><td>").expect("row time cell");
        let (vers, r) = r.split_once("</td><td>").expect("row version cell");
        let modified = match vers {
            "Original" => false,
            "<em>Modified" => true,
            other => panic!("version cell: {other:?}"),
        };
        let (origin, r) = r.split_once("</td></tr>").expect("row origin cell");
        rows.push(TheoryRow {
            index: idx.parse().expect("numeric row index"),
            name: unescape_entities(name),
            time: unescape_entities(time),
            modified,
            origin: unescape_entities(origin),
        });
        rest = r;
    }
    Welcome {
        version: unescape_entities(version),
        banner,
        rows,
    }
}

/// All captured index pages — GET on two servers (1-row and 3-version
/// tables), upload success (Loaded flash + uploaded-filename origin row),
/// POST without a file (Failed flash), a parse-failure upload (multi-line
/// escaped Custom flash), and a metachar-filename upload (row-origin
/// escaping) — re-rendered byte-identically [L19]–[L21].
#[test]
fn live_index_page_replays() {
    let pages = [
        ("index_3134_one_theory.html", Banner::None, 1),
        ("index_3135_three_versions.html", Banner::None, 3),
        ("index_post_success.html", Banner::Loaded, 2),
        ("index_post_nofile.html", Banner::Failed, 2),
        ("index_post_metachar.html", Banner::Loaded, 3),
    ];
    for (file, want_banner, want_rows) in pages {
        let page = r4(file);
        let w = invert_index(&page);
        assert_eq!(
            std::mem::discriminant(&w.banner),
            std::mem::discriminant(&want_banner),
            "{file}: banner kind"
        );
        assert_eq!(w.rows.len(), want_rows, "{file}: row count");
        assert_eq!(render_welcome(&w), page, "{file}: byte replay");
    }
}

/// The parse-failure upload's flash is a Custom multi-line message,
/// entity-escaped by the producer (raw quotes and newlines in the input)
/// [L20].
#[test]
fn live_index_custom_flash() {
    let page = r4("index_post_failure.html");
    let w = invert_index(&page);
    match &w.banner {
        Banner::Custom(m) => {
            assert_eq!(
                m,
                "Theory loading failed:\n\"garbage.spthy\" (line 1, column 1):\nunexpected \"t\"\nexpecting \"theory\""
            );
        }
        _ => panic!("expected a custom flash"),
    }
    assert_eq!(render_welcome(&w), page, "byte replay");
}

/// The metachar-filename upload pins row-origin entity escaping [L21].
#[test]
fn live_index_metachar_origin() {
    let page = r4("index_post_metachar.html");
    let w = invert_index(&page);
    assert_eq!(w.rows[2].origin, "up&<x>'probe.spthy");
    assert!(page.contains("<td>up&amp;&lt;x&gt;&#39;probe.spthy</td>"));
}

/// Plain-text housekeeping bodies, byte-exact vs the live captures [L19].
#[test]
fn housekeeping_bodies() {
    assert_eq!(ROBOTS_BODY, r4("robots.body"));
    assert_eq!(CANCEL_ACK_BODY, r4("cancel_ack.body"));
    assert_eq!(FILE_NOT_FOUND_BODY, r4("file_not_found.body"));
}

/// The Invalid-Arguments page (`GET /kill` without a path), byte-exact vs
/// the live capture [L19].
#[test]
fn invalid_args_page() {
    assert_eq!(
        render_invalid_args(&["No path to kill specified!".to_string()]),
        r4("kill_no_path.html")
    );
}

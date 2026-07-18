//! Capture-corpus sweep — SPEC acceptance ladder rung 2 for R1.
//!
//! For every `main/message` / `main/rules` / `main/tactic` / `main/help`
//! response captured across the 81 crawl manifests, this test:
//!   1. reads the RAW envelope bytes (materialized by
//!      workspace/tools/extract_r1_raw.py into workspace/r1_raw/);
//!   2. decodes the `{html,title}` envelope and INVERTS the producer's skin —
//!      un-postprocessing the fragment into logical lines and slicing the
//!      opaque content out of the observed block skeleton;
//!   3. feeds the sliced content back through the producer and asserts the
//!      re-rendered response equals the captured bytes EXACTLY.
//!
//! This exercises the frame + postprocess + envelope over the whole corpus
//! without a prover. A second test replays the curated round-1 byte targets
//! (producers/round1/targets/) the same way.
//!
//! The inversion is intentionally strict (panics on any unexpected shape) so
//! a capture that deviates from the modeled skeleton fails loudly instead of
//! being skipped.

use std::fs;
use std::path::PathBuf;

use producers_clean::model::{Content, ContentPane, EmptyRender, HeadedBlock, HelpPane};
use producers_clean::section::HELP_STATIC_HTML;
use producers_clean::{render_content_pane, render_help_pane};

fn workspace_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

// ---------------------------------------------------------------------------
// Envelope decoding (strict: asserts the exact observed skeleton [S07][S08])
// ---------------------------------------------------------------------------

/// Decode a JSON string starting after its opening quote; returns the decoded
/// text and the byte offset of the closing quote.
fn decode_json_string(s: &str) -> (String, usize) {
    let bytes = s.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    loop {
        match bytes[i] {
            b'"' => return (out, i),
            b'\\' => {
                i += 1;
                match bytes[i] {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'n' => out.push('\n'),
                    b't' => out.push('\t'),
                    b'r' => out.push('\r'),
                    b'b' => out.push('\u{8}'),
                    b'f' => out.push('\u{c}'),
                    b'u' => {
                        let cp = u32::from_str_radix(&s[i + 1..i + 5], 16).unwrap();
                        i += 4;
                        out.push(char::from_u32(cp).expect("BMP escape"));
                    }
                    other => panic!("unexpected JSON escape \\{}", other as char),
                }
                i += 1;
            }
            _ => {
                let c = s[i..].chars().next().unwrap();
                out.push(c);
                i += c.len_utf8();
            }
        }
    }
}

/// Split a raw `{"html":"…","title":"…"}` body into (html, title), asserting
/// the exact serialization skeleton.
fn decode_envelope(raw: &str) -> (String, String) {
    let rest = raw.strip_prefix("{\"html\":\"").expect("envelope opens with html key");
    let (html, end) = decode_json_string(rest);
    let rest = rest[end..]
        .strip_prefix("\",\"title\":\"")
        .expect("title key follows html");
    let (title, end) = decode_json_string(rest);
    assert_eq!(&rest[end..], "\"}", "envelope closes after title");
    (html, title)
}

// ---------------------------------------------------------------------------
// Skin inversion: postprocessed fragment -> logical lines -> headed blocks
// ---------------------------------------------------------------------------

/// Invert the per-line postprocess: `<br/>\n`-terminated lines with leading
/// `&nbsp;` runs back to plain lines with leading spaces.
fn unpostprocess(html: &str) -> Vec<String> {
    assert!(html.ends_with("<br/>\n"), "postprocessed fragment ends with a break");
    let mut lines: Vec<&str> = html.split("<br/>\n").collect();
    assert_eq!(lines.pop(), Some(""), "trailing separator");
    lines
        .iter()
        .map(|line| {
            let mut rest = *line;
            let mut n = 0;
            while let Some(r) = rest.strip_prefix("&nbsp;") {
                rest = r;
                n += 1;
            }
            format!("{}{}", " ".repeat(n), rest)
        })
        .collect()
}

const P_OPEN: &str = "<p class=\"monospace rules\">";

/// Parse logical lines into (leading-blank-line?, [(heading, body-lines)]).
fn parse_blocks(lines: &[String]) -> (bool, Vec<(String, Vec<String>)>) {
    let mut i = 0;
    let leading_blank = lines.first().is_some_and(|l| l.is_empty());
    if leading_blank {
        i = 1;
    }
    let mut blocks = Vec::new();
    while i < lines.len() {
        let heading = lines[i]
            .strip_prefix("<h2>")
            .and_then(|h| h.strip_suffix("</h2>"))
            .unwrap_or_else(|| panic!("expected heading line, got {:?}", lines[i]));
        i += 1;
        let first = lines[i]
            .strip_prefix(P_OPEN)
            .unwrap_or_else(|| panic!("expected paragraph open, got {:?}", lines[i]));
        let mut body = Vec::new();
        let mut cur = first.to_string();
        loop {
            if let Some(stripped) = cur.strip_suffix("</p>") {
                if !(stripped.is_empty() && body.is_empty()) {
                    body.push(stripped.to_string());
                }
                i += 1;
                break;
            }
            body.push(cur);
            i += 1;
            cur = lines[i].clone();
        }
        blocks.push((heading.to_string(), body));
    }
    (leading_blank, blocks)
}

fn content(body: Vec<String>) -> Content {
    Content { lines: body }
}

fn block(heading: &str, body: Vec<String>, when_empty: EmptyRender) -> HeadedBlock {
    HeadedBlock {
        heading: heading.into(),
        body: content(body),
        when_empty,
    }
}

// ---------------------------------------------------------------------------
// Per-family reconstruction
// ---------------------------------------------------------------------------

fn rebuild_message(html: &str, title: &str) -> ContentPane {
    assert_eq!(title, "Message theory");
    let (leading_blank, mut blocks) = parse_blocks(&unpostprocess(html));
    assert!(!leading_blank, "message pane has no leading blank");
    assert_eq!(blocks.len(), 3, "message pane emits exactly three sections");
    let heads: Vec<&str> = blocks.iter().map(|(h, _)| h.as_str()).collect();
    assert_eq!(
        heads,
        ["Signature", "Construction Rules", "Deconstruction Rules"]
    );
    ContentPane {
        title: title.into(),
        blocks: blocks
            .drain(..)
            .map(|(h, b)| block(&h, b, EmptyRender::Keep))
            .collect(),
    }
}

fn rebuild_rules(html: &str, title: &str) -> ContentPane {
    assert_eq!(title, "Multiset rewriting rules and restrictions");
    let (leading_blank, parsed) = parse_blocks(&unpostprocess(html));
    let mut iter = parsed.into_iter().peekable();

    // Macros slot: a real block, or the leading blank line when absent [L03].
    let macros_body = if iter.peek().is_some_and(|(h, _)| h == "Macros") {
        assert!(!leading_blank);
        iter.next().unwrap().1
    } else {
        assert!(leading_blank, "no macros block implies the blank slot");
        Vec::new()
    };
    let mut blocks = vec![block("Macros", macros_body, EmptyRender::BlankLine)];

    let (h, b) = iter.next().expect("injective-facts section");
    assert_eq!(h, "Fact Symbols with Injective Instances");
    blocks.push(block(&h, b, EmptyRender::Keep));

    let (h, b) = iter.next().expect("MSR section");
    assert_eq!(h, "Multiset Rewriting Rules");
    blocks.push(block(&h, b, EmptyRender::Keep));

    // Restrictions: present, or omitted without residue [S10][S11].
    let restrictions = match iter.next() {
        Some((h, b)) => {
            assert_eq!(h, "Restrictions of the Set of Traces");
            b
        }
        None => Vec::new(),
    };
    blocks.push(block(
        "Restrictions of the Set of Traces",
        restrictions,
        EmptyRender::Omit,
    ));
    assert!(iter.next().is_none(), "no further sections");
    ContentPane {
        title: title.into(),
        blocks,
    }
}

fn rebuild_tactic(html: &str, title: &str) -> ContentPane {
    assert_eq!(title, "Tactics");
    let (leading_blank, mut blocks) = parse_blocks(&unpostprocess(html));
    assert!(!leading_blank);
    assert_eq!(blocks.len(), 1);
    let (h, b) = blocks.pop().unwrap();
    assert_eq!(h, "Tactic(s)");
    ContentPane {
        title: title.into(),
        blocks: vec![block(&h, b, EmptyRender::Keep)],
    }
}

/// Reverse of `escape_text` for the env-line fields (strict).
fn unescape_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        let (c, len) = if rest.starts_with("&amp;") {
            ('&', 5)
        } else if rest.starts_with("&lt;") {
            ('<', 4)
        } else if rest.starts_with("&gt;") {
            ('>', 4)
        } else if rest.starts_with("&quot;") {
            ('"', 6)
        } else if rest.starts_with("&#39;") {
            ('\'', 5)
        } else {
            panic!("unknown entity at {:?}", &rest[..rest.len().min(8)]);
        };
        out.push(c);
        rest = &rest[len..];
    }
    out.push_str(rest);
    out
}

fn rebuild_help(html: &str, title: &str) -> HelpPane {
    let static_at = html.find("<div id=\"help\">").expect("static help block");
    assert_eq!(&html[static_at..], HELP_STATIC_HTML, "invariant static block");
    let env = html[..static_at]
        .strip_prefix("<p>Theory: ")
        .and_then(|e| e.strip_suffix("</p>"))
        .expect("env line shape");
    let (pre, banner) = match env.find("<div class=\"wf-warning\"") {
        Some(i) => (&env[..i], &env[i..]),
        None => (env, ""),
    };
    let pre = pre.strip_suffix(") ").expect("env line closes with `) `");
    let (name, rest) = pre.split_once(" (Loaded at ").expect("Loaded-at marker");
    let (time, origin) = rest.split_once(" from ").expect("from marker");
    let name = unescape_entities(name);
    assert_eq!(title, format!("Theory: {name}"));
    HelpPane {
        theory_name: name,
        load_time: unescape_entities(time),
        origin: unescape_entities(origin),
        wf_banner_html: banner.into(),
    }
}

fn replay(family: &str, raw: &str) -> String {
    let (html, title) = decode_envelope(raw);
    match family {
        "message" => render_content_pane(&rebuild_message(&html, &title)),
        "rules" => render_content_pane(&rebuild_rules(&html, &title)),
        "tactic" => render_content_pane(&rebuild_tactic(&html, &title)),
        "help" => render_help_pane(&rebuild_help(&html, &title)),
        other => panic!("unknown family {other}"),
    }
}

// ---------------------------------------------------------------------------
// The sweeps
// ---------------------------------------------------------------------------

/// All 81 manifests x 4 center fragments: slice, re-render, byte-compare the
/// whole response body (envelope included).
#[test]
fn corpus_sweep_all_manifests() {
    let dir = workspace_path("../r1_raw");
    let mut count = 0;
    let mut per_family = [0usize; 4];
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .expect("workspace/r1_raw materialized (tools/extract_r1_raw.py)")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "raw"))
        .collect();
    entries.sort();
    for path in entries {
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let family = stem.rsplit("__main_").next().unwrap();
        let raw = fs::read_to_string(&path).unwrap();
        let rendered = replay(family, &raw);
        assert_eq!(rendered, raw, "byte mismatch replaying {stem}");
        per_family[["message", "rules", "tactic", "help"]
            .iter()
            .position(|f| *f == family)
            .unwrap()] += 1;
        count += 1;
    }
    assert_eq!(count, 324, "81 manifests x 4 fragments");
    assert_eq!(per_family, [81; 4]);
}

/// Live-probe replays (workspace/r1_live/, raw bodies captured from the
/// oracle server on theories NOT in the crawl corpus): the own-authored
/// metachar-filename EscProbe theory [L06] and the macros-bearing
/// MacroGlobalVarNSPK3 [L03] — the only observed macros-present rules pane.
#[test]
fn live_probe_replays() {
    let dir = workspace_path("../r1_live");
    let mut count = 0;
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .expect("workspace/r1_live materialized (QUERIES.log [L03][L06][L07])")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "raw"))
        .collect();
    entries.sort();
    for path in entries {
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let family = stem.rsplit("__main_").next().unwrap();
        let raw = fs::read_to_string(&path).unwrap();
        assert_eq!(replay(family, &raw), raw, "byte mismatch replaying {stem}");
        count += 1;
    }
    assert_eq!(count, 8, "2 live theories x 4 fragments");
}

/// The curated round-1 byte targets (producers/round1/targets/): decoded
/// fragment + title files; re-render and byte-compare fragment and title.
#[test]
fn round1_materialized_targets() {
    let dir = workspace_path("../../round1/targets");
    let mut count = 0;
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .expect("round1/targets materialized")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    entries.sort();
    for path in entries {
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let family = stem.rsplit("_main_").next().unwrap();
        let html = fs::read_to_string(&path).unwrap();
        let title = fs::read_to_string(path.with_extension("title")).unwrap();
        let rebuilt_envelope = match family {
            "message" => render_content_pane(&rebuild_message(&html, &title)),
            "rules" => render_content_pane(&rebuild_rules(&html, &title)),
            "tactic" => render_content_pane(&rebuild_tactic(&html, &title)),
            "help" => render_help_pane(&rebuild_help(&html, &title)),
            other => panic!("unknown family {other}"),
        };
        let (out_html, out_title) = decode_envelope(&rebuilt_envelope);
        assert_eq!(out_html, html, "fragment byte mismatch for {stem}");
        assert_eq!(out_title, title, "title mismatch for {stem}");
        count += 1;
    }
    assert_eq!(count, 44, "11 curated labels x 4 fragments");
}

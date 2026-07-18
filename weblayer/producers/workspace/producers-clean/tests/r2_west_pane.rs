//! R2 acceptance — the proof-script WEST pane assembly.
//!
//! For every overview capture across the 81 manifests (478 bodies: 396 proof
//! views + 82 help views, materialized by tools/extract_r2_panes.py into
//! workspace/r2_panes/), this test:
//!   1. reads the pane inner HTML (the proof-script container's content);
//!   2. INVERTS the producer's skin — un-postprocessing into logical lines and
//!      slicing the opaque content (item labels/annotations, lemma names,
//!      attribute text, formula lines, proof-display lines) out of the strict
//!      observed skeleton, with every frame byte asserted on the way;
//!   3. feeds the slices back through `render_proof_script` and asserts the
//!      re-rendered pane equals the captured bytes EXACTLY.
//!
//! A second test replays live-captured panes from theories NOT in the corpus
//! (workspace/r2_live/, QUERIES.log [L15]) including a proved state, pinning
//! the inline-layout width boundary at exactly 69/70 ([L14]).
//!
//! The inversion is intentionally strict (panics on any unexpected shape) so
//! a capture deviating from the modeled skeleton fails loudly.

use std::fs;
use std::path::PathBuf;

use producers_clean::model::{
    Content, LemmaEntry, NavItem, ProofDisplay, ProofScriptPane, ThyPath,
};
use producers_clean::{parse_path, render_proof_script};

fn workspace_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

// ---------------------------------------------------------------------------
// Line-level inversion helpers
// ---------------------------------------------------------------------------

/// Invert the per-line postprocess: split on `<br/>\n`, leading `&nbsp;` runs
/// back to spaces; assert the pane's final ` ` frame byte.
fn pane_lines(inner: &str) -> Vec<String> {
    let doc = inner
        .strip_suffix(' ')
        .expect("pane ends with the trailing space");
    assert!(doc.ends_with("<br/>\n"), "pane lines end with breaks");
    let mut lines: Vec<&str> = doc.split("<br/>\n").collect();
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

/// Reverse of the producer's entity escape (strict).
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

/// Escaped width as the layout rule measures it (tags 0, all else 1/char).
fn escaped_width(line: &str) -> usize {
    let mut w = 0;
    let mut in_tag = false;
    for c in line.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => w += 1,
            _ => {}
        }
    }
    w
}

const KW_LEMMA: &str = "<span class=\"hl_keyword\">lemma</span> ";
const KW_END: &str = "<span class=\"hl_keyword\">end</span>";

// ---------------------------------------------------------------------------
// Pane inversion
// ---------------------------------------------------------------------------

fn slice_pane(inner: &str) -> (u64, ProofScriptPane) {
    let lines = pane_lines(inner);
    let mut i = 0;

    // 1. Header line.
    let rest = lines[0]
        .strip_prefix("<span class=\"hl_keyword\">theory</span> <a class=\"internal-link help\" href=\"/thy/trace/")
        .expect("theory header line");
    let (idx_str, rest) = rest.split_once("/main/help\">").expect("help href");
    let idx: u64 = idx_str.parse().expect("numeric index");
    let name = rest
        .strip_suffix("</a> <span class=\"hl_keyword\">begin</span>")
        .expect("begin keyword");
    let theory_name = unescape_entities(name);
    i += 1;

    // 2. Item link lines (blank + item), until the add-first link.
    let mut items = Vec::new();
    loop {
        assert_eq!(lines[i], "", "blank before item/add line");
        if lines[i + 1].starts_with("<a class=\"internal-link add\"") {
            break;
        }
        let it = lines[i + 1]
            .strip_prefix(&format!("<a class=\"internal-link\" href=\"/thy/trace/{idx}/main/"))
            .unwrap_or_else(|| panic!("item line shape: {:?}", lines[i + 1]));
        let (tail, it) = it.split_once("\"><strong>").expect("item strong open");
        let (label, it) = it.split_once("</strong> ").expect("item strong close");
        let annotation = it.strip_suffix("</a>").expect("item anchor close");
        let target = parse_path(tail).expect("item target parses");
        // The link is canonical: R5 re-renders it byte-identically.
        items.push(NavItem {
            target,
            label: label.to_string(),
            annotation: annotation.to_string(),
        });
        i += 2;
    }
    // 3. The add-first link.
    assert_eq!(
        lines[i + 1],
        format!("<a class=\"internal-link add\" href=\"/thy/trace/{idx}/main/add/%3Cfirst%3E\">add lemma</a>"),
        "add-first link"
    );
    i += 2;

    // 4. Lemma blocks; 5. end.
    let mut lemmas = Vec::new();
    loop {
        assert_eq!(lines[i], "", "blank before lemma/end");
        i += 1;
        if lines[i] == KW_END {
            assert_eq!(i, lines.len() - 1, "end is the last line");
            break;
        }
        if lines[i].is_empty() && lines[i + 1] == KW_END {
            // Zero-lemma pane: two blanks before `end`.
            assert!(lemmas.is_empty());
            assert_eq!(i + 1, lines.len() - 1, "end is the last line");
            break;
        }

        // 4a. Declaration line(s), with optional status wrapper.
        let (wrapper, decl_rest) = match lines[i].strip_prefix(KW_LEMMA) {
            Some(rest) => (None, rest.to_string()),
            None => {
                let w = lines[i]
                    .strip_prefix("<span class=\"")
                    .unwrap_or_else(|| panic!("decl line shape: {:?}", lines[i]));
                let (class, rest) = w.split_once("\">").expect("wrapper span open");
                let rest = rest.strip_prefix(KW_LEMMA).expect("lemma keyword after wrapper");
                (Some(class.to_string()), rest.to_string())
            }
        };
        let mut decl = decl_rest;
        while !decl.ends_with(':') {
            i += 1;
            decl.push('\n');
            decl.push_str(&lines[i]);
        }
        i += 1;
        let decl = decl.strip_suffix(':').unwrap();
        let (name_html, attributes) = match decl.find(" [") {
            Some(p) => (&decl[..p], &decl[p..]),
            None => (decl, ""),
        };
        let name = unescape_entities(name_html);

        // 4b. Quantifier / formula block at indent 2.
        let q = lines[i]
            .strip_prefix("  ")
            .unwrap_or_else(|| panic!("quantifier indent: {:?}", lines[i]));
        let (quantifier, formula_lines) = if let Some((kw, rest)) = q.split_once(' ') {
            assert!(matches!(kw, "all-traces" | "exists-trace"), "quantifier {kw:?}");
            // Inline layout: the assembled line obeys the width limit.
            assert!(
                escaped_width(&lines[i]) <= 69,
                "inline line wider than 69: {:?}",
                lines[i]
            );
            i += 1;
            (kw.to_string(), vec![rest.to_string()])
        } else {
            assert!(matches!(q, "all-traces" | "exists-trace"), "quantifier {q:?}");
            let kw = q.to_string();
            i += 1;
            let mut fl = Vec::new();
            while !lines[i].starts_with("<a class=\"internal-link edit\"") {
                let l = lines[i]
                    .strip_prefix("  ")
                    .unwrap_or_else(|| panic!("formula indent: {:?}", lines[i]));
                fl.push(l.to_string());
                i += 1;
            }
            // Vertical layout is justified: multi-line, or over the limit.
            if let [only] = fl.as_slice() {
                assert!(
                    escaped_width(&format!("  {kw} {only}")) > 69,
                    "single-line formula under the limit rendered vertically"
                );
            }
            (kw, fl)
        };

        // 4c. Edit-or-delete line (closes the wrapper span when present).
        let e = lines[i]
            .strip_prefix(&format!("<a class=\"internal-link edit\" href=\"/thy/trace/{idx}/main/edit/"))
            .unwrap_or_else(|| panic!("edit line shape: {:?}", lines[i]));
        let (enc_name, e) = e.split_once("\">edit lemma</a>  or  ").expect("or separator");
        let e = e
            .strip_prefix(&format!("<a class=\"internal-link delete\" href=\"/thy/trace/{idx}/main/delete/"))
            .expect("delete anchor");
        let (_enc2, e) = e.split_once("\">delete lemma</a>").expect("delete anchor close");
        match wrapper {
            Some(_) => assert_eq!(e, "</span>", "wrapper closes after delete anchor"),
            None => assert_eq!(e, "", "no wrapper close on a bare header"),
        }
        i += 1;

        // 4d. Proof display: the exact sorry step, or opaque pre-rendered lines.
        let sorry_line = format!(
            "<span class=\"hl_keyword\">by</span> <a class=\"internal-link proof-step sorry-step\" href=\"/thy/trace/{idx}/main/proof/{enc_name}\"><span class=\"hl_keyword\">sorry</span></a>"
        );
        let proof = if lines[i] == sorry_line {
            assert!(wrapper.is_none(), "sorry proof has no header wrapper");
            i += 1;
            ProofDisplay::Unproven
        } else {
            let mut pl = Vec::new();
            while !lines[i].is_empty() {
                pl.push(lines[i].clone());
                i += 1;
            }
            ProofDisplay::Rendered {
                header_status: wrapper,
                lines: pl,
            }
        };

        // 4e. Blank + this lemma's add link.
        assert_eq!(lines[i], "", "blank after proof display");
        assert_eq!(
            lines[i + 1],
            format!("<a class=\"internal-link add\" href=\"/thy/trace/{idx}/main/add/{enc_name}\">add lemma</a>"),
            "per-lemma add link"
        );
        i += 2;

        lemmas.push(LemmaEntry {
            name,
            attributes: attributes.to_string(),
            quantifier,
            formula: Content { lines: formula_lines },
            proof,
        });
    }

    (
        idx,
        ProofScriptPane {
            theory_name,
            index: idx,
            items,
            lemmas,
        },
    )
}

fn replay(inner: &str) -> String {
    let (_idx, pane) = slice_pane(inner);
    // Structural expectations that hold corpus-wide ([S16]): five nav items,
    // message first, both sources views present.
    assert_eq!(pane.items.len(), 5, "five nav items");
    assert_eq!(pane.items[0].target, ThyPath::Message);
    render_proof_script(&pane)
}

// ---------------------------------------------------------------------------
// The sweeps
// ---------------------------------------------------------------------------

/// All 478 overview panes (82 help + 396 proof views) across the 81
/// manifests: slice, re-render, byte-compare the pane.
#[test]
fn corpus_sweep_all_overview_panes() {
    let dir = workspace_path("../r2_panes");
    let mut count = 0;
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .expect("workspace/r2_panes materialized (tools/extract_r2_panes.py)")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "pane"))
        .collect();
    entries.sort();
    for path in entries {
        let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let inner = fs::read_to_string(&path).unwrap();
        assert_eq!(replay(&inner), inner, "byte mismatch replaying {stem}");
        count += 1;
    }
    assert_eq!(count, 478, "82 help + 396 proof views");
}

/// Live-probe replays (workspace/r2_live/): panes captured from the oracle on
/// theories NOT in the crawl corpus — the own-authored PathProbe (fresh) and
/// WProbe (the 35-lemma inline/vertical width bisection theory, straddling
/// the 69/70 boundary on four formula families), plus PathProbe at version 2
/// after a live autoprove (a proved hl_good tree) [L14][L15].
#[test]
fn live_probe_pane_replays() {
    let dir = workspace_path("../r2_live");
    let mut count = 0;
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .expect("workspace/r2_live materialized (QUERIES.log [L15])")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "pane"))
        .collect();
    entries.sort();
    for path in entries {
        let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let inner = fs::read_to_string(&path).unwrap();
        assert_eq!(replay(&inner), inner, "byte mismatch replaying {stem}");
        count += 1;
    }
    assert_eq!(count, 3, "PathProbe v1 + WProbe v1 + proved PathProbe v2");
}

/// Constructed-input fixture: a two-lemma pane with one proved lemma, pinning
/// the frame bytes against hand-checked observed shapes (the AttestedComputation
/// west pane's element order, [S16]).
#[test]
fn fixture_minimal_pane() {
    let pane = ProofScriptPane {
        theory_name: "T".into(),
        index: 3,
        items: vec![NavItem {
            target: ThyPath::Message,
            label: "Message theory".into(),
            annotation: "".into(),
        }],
        lemmas: vec![LemmaEntry {
            name: "foo".into(),
            attributes: " [reuse]".into(),
            quantifier: "all-traces".into(),
            formula: Content { lines: vec!["&quot;F&quot;".into()] },
            proof: ProofDisplay::Unproven,
        }],
    };
    let got = render_proof_script(&pane);
    let want = "<span class=\"hl_keyword\">theory</span> <a class=\"internal-link help\" href=\"/thy/trace/3/main/help\">T</a> <span class=\"hl_keyword\">begin</span><br/>\n\
        <br/>\n\
        <a class=\"internal-link\" href=\"/thy/trace/3/main/message\"><strong>Message theory</strong> </a><br/>\n\
        <br/>\n\
        <a class=\"internal-link add\" href=\"/thy/trace/3/main/add/%3Cfirst%3E\">add lemma</a><br/>\n\
        <br/>\n\
        <span class=\"hl_keyword\">lemma</span> foo [reuse]:<br/>\n\
        &nbsp;&nbsp;all-traces &quot;F&quot;<br/>\n\
        <a class=\"internal-link edit\" href=\"/thy/trace/3/main/edit/foo\">edit lemma</a>  or  <a class=\"internal-link delete\" href=\"/thy/trace/3/main/delete/foo\">delete lemma</a><br/>\n\
        <span class=\"hl_keyword\">by</span> <a class=\"internal-link proof-step sorry-step\" href=\"/thy/trace/3/main/proof/foo\"><span class=\"hl_keyword\">sorry</span></a><br/>\n\
        <br/>\n\
        <a class=\"internal-link add\" href=\"/thy/trace/3/main/add/foo\">add lemma</a><br/>\n\
        <br/>\n\
        <span class=\"hl_keyword\">end</span><br/>\n ";
    assert_eq!(got, want);
    // Round-trip through the inverter too.
    assert_eq!(replay_unchecked(&got), got);
}

/// Zero-lemma pane: two blank lines between the add-first link and `end`
/// (observed in the two lemma-less corpus panes, [S16]).
#[test]
fn fixture_zero_lemma_spacing() {
    let pane = ProofScriptPane {
        theory_name: "E".into(),
        index: 1,
        items: vec![],
        lemmas: vec![],
    };
    let got = render_proof_script(&pane);
    assert!(got.ends_with(
        "add lemma</a><br/>\n<br/>\n<br/>\n<span class=\"hl_keyword\">end</span><br/>\n "
    ));
}

/// Replay without the five-item structural assertion (fixtures use fewer).
fn replay_unchecked(inner: &str) -> String {
    let (_idx, pane) = slice_pane(inner);
    render_proof_script(&pane)
}

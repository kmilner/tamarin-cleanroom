//! R5 acceptance — the theory-path grammar.
//!
//! Three layers:
//!  1. live-probe replays: the parse ACCEPTANCE battery observed against the
//!     oracle server (QUERIES.log [L08]–[L12]) — every accepted probe parses,
//!     every 404 probe returns `None`;
//!  2. parse ⇄ render round-trip laws on constructed values;
//!  3. the corpus byte sweep: every DISTINCT `main/*` href tail harvested from
//!     the 81 capture manifests (workspace/r5_tails/tails.txt, [S15]) parses
//!     and re-renders byte-identically.

use std::fs;
use std::path::PathBuf;

use producers_clean::model::ThyPath;
use producers_clean::{parse_path, render_path};

fn workspace_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn tail(p: &ThyPath) -> String {
    render_path(p).join("/")
}

// ---------------------------------------------------------------------------
// 1. Live-probe replays (parse decisions pinned by the oracle batteries)
// ---------------------------------------------------------------------------

/// Paths the live server ACCEPTED (200 through the theory-path grammar).
#[test]
fn live_accepted_paths_parse() {
    let accepted = [
        // [L08] heads
        "help",
        "message",
        "rules",
        "tactic",
        "lemma/foo",
        "lemma/nonexistent", // resolution, not parse
        "proof/foo",
        "proof/foo/_",
        "proof/nonexistent",
        "cases/raw/0/0",
        "cases/refined/0/0",
        // [L08]/[L10] numeric leniency
        "cases/raw/00/0",
        "cases/raw/0/00",
        "cases/raw/-1/0",
        "cases/raw/0x1/0",
        "cases/raw/0/0/extra",
        "cases/raw/9/9",
        "cases/raw/0o10/0",
        "cases/raw/0b1/0",
        "cases/raw/0X1/0",
        "cases/raw/0O10/0",
        "cases/raw/(1)/0",
        "cases/raw/((1))/0",
        "cases/raw/-%201/0",
        "cases/raw/(%20-1%20)/0",
        "cases/raw/1_/0",
        "cases/raw/1_0/0",
        "cases/raw/0x_1/0",
        "cases/raw/%091%09/0",
        "cases/raw/%201/0",
        "cases/raw/1%20/0",
        "cases/raw/1abc/0",
        "cases/raw/1-/0",
        "cases/raw/1)/0",
        "cases/raw/(1)x/0",
        "cases/raw/99999999999999999999/0",
        // [L09] decoding + extra-segment tolerance
        "me%73sage",
        "message/",
        "message/extra",
        "help/extra",
        "add/%3Cfirst%3E",
        "add/foo",
        "add/sp%20ace",
        "add/pct%25enc",
        "add/foo/extra",
        "add/a+b",
        "add/a%2Bb",
        "add/100%25",
        "add/a%zzb",
        "add/a%",
        "add/a%2",
        "add/a%G1b",
        "add/caf%C3%A9",
        "edit/foo",
        "edit/nonexistent",
        "delete/foo",
        "proof/foo%2F_",
        "proof/foo/B_2",
        // [L11] empty names parse
        "edit/",
        "proof/",
        "proof//_",
        "proof/foo/",
        "lemma/",
        "lemma/foo/extra",
    ];
    for p in accepted {
        assert!(parse_path(p).is_some(), "server accepted {p:?} but parse_path rejected it");
    }
}

/// Paths the live server rejected with the 404 surface.
#[test]
fn live_rejected_paths_return_none() {
    let rejected = [
        // [L08] arity + head spelling
        "proof",
        "cases/raw/1",
        "cases/bogus/0/0",
        "cases/RAW/0/0",
        "MESSAGE",
        "/message", // leading empty segment ([L09] main//message)
        // [L10] numeric rejections
        "cases/raw/abc/0",
        "cases/raw/0/abc",
        "cases/raw/+1/0",
        "cases/raw/_1/0",
        "cases/raw/1.0/0",
        "cases/raw/1e2/0",
        "cases/raw/--1/0",
        "cases/raw/-(-1)/0",
        "cases/raw/(-1/0",
        "cases/raw/(1x)/0",
        "cases/raw//0",
        "cases/raw/%E2%88%80/0",
        // [L11] arity minima + non-heads
        "add",
        "edit",
        "delete",
        "lemma",
        "cases/raw",
        "cases",
        "",
        "sources",
        // out of the producer link vocabulary (interface contract; the server
        // itself accepts method/{lemma}/{n} [L11])
        "method/foo/1",
    ];
    for p in rejected {
        assert_eq!(parse_path(p), None, "parse_path accepted {p:?}");
    }
}

/// Decoded values pinned by the add-form echo channel ([L09][L12]).
#[test]
fn decoding_matches_observed_echoes() {
    let cases = [
        ("add/sp%20ace", "sp ace"),
        ("add/pct%25enc", "pct%enc"),
        ("add/a+b", "a+b"),      // '+' is NOT a space in a path segment
        ("add/a%2Bb", "a+b"),
        ("add/100%25", "100%"),
        ("add/a%zzb", "a%zzb"),  // invalid escapes stay literal
        ("add/a%", "a%"),
        ("add/a%2", "a%2"),
        ("add/a%G1b", "a%G1b"),
        ("add/caf%C3%A9", "café"),
        ("add/a%FFb", "a\u{FFFD}b"), // invalid UTF-8 -> replacement
        ("add/a%C3b", "a\u{FFFD}b"),
        ("add/%3Cfirst%3E", "<first>"),
    ];
    for (raw, want) in cases {
        match parse_path(raw) {
            Some(ThyPath::Add(pos)) => assert_eq!(pos, want, "decoding {raw:?}"),
            other => panic!("{raw:?} parsed to {other:?}"),
        }
    }
    // %2F decodes into the NAME, it does not split ([L09]).
    assert_eq!(
        parse_path("proof/foo%2F_"),
        Some(ThyPath::Proof { lemma: "foo/_".into(), sub: vec![] })
    );
}

// ---------------------------------------------------------------------------
// 2. Round-trip laws
// ---------------------------------------------------------------------------

#[test]
fn parse_render_round_trip() {
    let values = [
        ThyPath::Help,
        ThyPath::Message,
        ThyPath::Rules,
        ThyPath::Tactic,
        ThyPath::Sources { refined: false, source_idx: 0, case_idx: 0 },
        ThyPath::Sources { refined: true, source_idx: 3, case_idx: 12 },
        ThyPath::Lemma("foo".into()),
        ThyPath::Lemma("".into()),
        ThyPath::Lemma("with space".into()),
        ThyPath::Lemma("café".into()),
        ThyPath::Proof { lemma: "exec".into(), sub: vec![] },
        ThyPath::Proof {
            lemma: "exec".into(),
            sub: vec!["_".into(), "B_2".into(), "case_1".into()],
        },
        ThyPath::Proof { lemma: "a/b".into(), sub: vec!["<x>".into()] },
        ThyPath::Edit("L1".into()),
        ThyPath::Add("<first>".into()),
        ThyPath::Add("lemma_name".into()),
        ThyPath::Delete("L1".into()),
    ];
    for v in values {
        let rendered = tail(&v);
        assert_eq!(parse_path(&rendered), Some(v.clone()), "round-trip via {rendered:?}");
    }
}

/// The one percent-escape the corpus renders: `<first>` -> `%3Cfirst%3E`
/// (uppercase hex, [S14]).
#[test]
fn add_first_link_bytes() {
    assert_eq!(
        render_path(&ThyPath::Add("<first>".into())),
        vec!["add".to_string(), "%3Cfirst%3E".to_string()]
    );
    assert_eq!(
        render_path(&ThyPath::Sources { refined: false, source_idx: 0, case_idx: 0 }),
        vec!["cases".to_string(), "raw".to_string(), "0".to_string(), "0".to_string()]
    );
}

// ---------------------------------------------------------------------------
// 3. Corpus byte sweep
// ---------------------------------------------------------------------------

/// Every distinct `main/*` href tail across the 81 manifests parses and
/// re-renders byte-identically. `method/…` tails are the documented
/// out-of-vocabulary family (497 of the 40037 distinct tails, [S14][S15]).
#[test]
fn corpus_tails_round_trip_byte_identical() {
    let file = workspace_path("../r5_tails/tails.txt");
    let data = fs::read_to_string(&file)
        .expect("workspace/r5_tails materialized (tools/extract_r5_tails.py)");
    let mut total = 0usize;
    let mut method = 0usize;
    for line in data.lines() {
        let (_count, tail_str) = line.split_once('\t').expect("count\\ttail");
        total += 1;
        if tail_str.starts_with("method/") {
            method += 1;
            assert_eq!(parse_path(tail_str), None, "method tail {tail_str:?} is out of vocabulary");
            continue;
        }
        let parsed = parse_path(tail_str)
            .unwrap_or_else(|| panic!("corpus tail failed to parse: {tail_str:?}"));
        assert_eq!(tail(&parsed), tail_str, "render bytes diverge for {tail_str:?}");
    }
    assert_eq!(total, 40037, "distinct corpus tails");
    assert_eq!(method, 497, "distinct method/ tails");
}

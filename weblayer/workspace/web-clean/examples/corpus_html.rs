//! Corpus-wide byte-parity harness for the two HTML page families.
//!
//! Reads an NDJSON extract of every `kind == "html"` response body across all
//! captured crawl manifests (one JSON object per line, produced by the jq
//! extractor documented in `workspace/REPORT2.md`), renders each body from the
//! crate's own templates using inputs taken from *observable* sources, and
//! reports per-family byte-exact reproduction counts.
//!
//! Run: `cargo run --release --example corpus_html -- <html_corpus.ndjson>`
//!
//! Each input record is `{mf, name, ver, file, u, b}`:
//! * `mf`   — manifest id (for diagnostics only)
//! * `name` — theory name, taken once per manifest from that manifest's
//!   `overview/help` `<title>` (a sibling artifact, not this body)
//! * `ver`  — Tamarin version, likewise from `overview/help`
//! * `file` — source filename, likewise from `overview/help`
//! * `u`    — the request URL key (index normalised to `#` by the capture tool)
//! * `b`    — the captured response body we must reproduce
//!
//! Non-circularity: for the `intdot` family the only value read back from the
//! target body `b` is the scalar request index (the `#` the capture erased); the
//! theory name comes from a sibling page and the path tail comes from the URL
//! key, so a byte match proves the template, name placement and tail-passthrough
//! independently. For the `overview` family the west/center pane inner HTML are
//! treated as opaque fragments (prover/proof-state output) exactly as the
//! committed page tests do; the harness measures whether the crate's shell
//! scaffolding + slot substitution reproduces the surrounding page bytes.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};

use serde::Deserialize;
use web_clean::intdot::{dotsrc_path, render_intdot};
use web_clean::page::{render_page, PageParams};

#[derive(Deserialize)]
struct Rec {
    #[allow(dead_code)]
    mf: String,
    name: String,
    ver: String,
    file: String,
    u: String,
    b: String,
}

/// Split `/thy/trace/#/<handler>/<tail>` into (handler, tail).
fn handler_and_tail(u: &str) -> Option<(&str, &str)> {
    let rest = u.strip_prefix("/thy/trace/#/")?;
    match rest.split_once('/') {
        Some((h, t)) => Some((h, t)),
        None => Some((rest, "")),
    }
}

/// Read the decimal index that follows `needle` in `body` (e.g. the request
/// index the capture tool normalised to `#`, recovered from an emitted link).
fn index_after(body: &str, needle: &str) -> Option<u64> {
    let start = body.find(needle)? + needle.len();
    let digits: String = body[start..].chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

/// Extract the substring of `body` strictly between `open` and the next `close`
/// occurring after it.
fn between<'a>(body: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let a = body.find(open)? + open.len();
    let b = body[a..].find(close)? + a;
    Some(&body[a..b])
}

// Fixed pane delimiters (observed; the same bytes the page shell uses).
const WEST_OPEN: &str = r#"<div class="monospace" id="proof">"#;
const WEST_CLOSE: &str = r#"</div></div></div><div class="ui-layout-east">"#;
const CENTER_OPEN: &str = r#"<div id="ui-main-display">"#;
const CENTER_CLOSE: &str = r#"</div></div></div><div id="dialog">"#;

#[derive(Default)]
struct Tally {
    total: usize,
    ok: usize,
    /// example failing URL (first seen)
    first_fail: Option<String>,
}
impl Tally {
    fn record(&mut self, ok: bool, u: &str) {
        self.total += 1;
        if ok {
            self.ok += 1;
        } else if self.first_fail.is_none() {
            self.first_fail = Some(u.to_string());
        }
    }
    fn pct(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            100.0 * self.ok as f64 / self.total as f64
        }
    }
}

fn overview_subfamily(tail: &str) -> &'static str {
    if tail == "help" {
        "overview/help"
    } else if tail.starts_with("proof/") {
        "overview/proof"
    } else {
        "overview/other"
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: corpus_html <ndjson>");
    let f = std::fs::File::open(&path).expect("open corpus");
    let reader = BufReader::new(f);

    let mut intdot = Tally::default();
    let mut intdot_name_guard_fail = 0usize;
    let mut intdot_tail_guard_fail = 0usize;
    let mut overview: BTreeMap<&'static str, Tally> = BTreeMap::new();
    // center-pane linkage sub-metric: does the center pane end in the observed
    // single trailing space that distinguishes it from the raw main/* html?
    let mut center_trailing_space = Tally::default();
    let mut other_handler: BTreeMap<String, usize> = BTreeMap::new();
    let mut parse_errors = 0usize;

    for line in reader.lines() {
        let line = line.expect("read line");
        if line.trim().is_empty() {
            continue;
        }
        let r: Rec = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => {
                parse_errors += 1;
                continue;
            }
        };
        let Some((handler, tail)) = handler_and_tail(&r.u) else {
            *other_handler.entry(r.u.clone()).or_default() += 1;
            continue;
        };

        match handler {
            "intdot" => {
                // idx: the one value read from the target (the erased request index).
                let idx = match index_after(&r.b, "dotsrc=\"/thy/trace/") {
                    Some(i) => i,
                    None => {
                        intdot.record(false, &r.u);
                        continue;
                    }
                };
                // Non-circularity guards (independent of the byte compare):
                //  - the name shown in the body matches the sibling-derived name
                //  - the path tail echoed in dotsrc matches the URL key tail
                if let Some(bn) = between(&r.b, "<title>Theory: ", "</title>") {
                    if bn != r.name {
                        intdot_name_guard_fail += 1;
                    }
                }
                if let Some(bt) = between(&r.b, "interactive-graph-def/", "\"") {
                    if bt != tail {
                        intdot_tail_guard_fail += 1;
                    }
                }
                let dotsrc = dotsrc_path(idx, tail);
                let got = render_intdot(&r.name, &dotsrc);
                intdot.record(got == r.b, &r.u);
            }
            "overview" => {
                let sub = overview_subfamily(tail);
                let idx = match index_after(&r.b, "action=\"/thy/trace/") {
                    Some(i) => i,
                    None => {
                        overview.entry(sub).or_default().record(false, &r.u);
                        continue;
                    }
                };
                let west = between(&r.b, WEST_OPEN, WEST_CLOSE);
                let center = between(&r.b, CENTER_OPEN, CENTER_CLOSE);
                let (west, center) = match (west, center) {
                    (Some(w), Some(c)) => (w, c),
                    _ => {
                        overview.entry(sub).or_default().record(false, &r.u);
                        continue;
                    }
                };
                let params = PageParams {
                    theory_name: &r.name,
                    index: idx,
                    version: &r.ver,
                    filename: &r.file,
                };
                let got = render_page(&params, west, center);
                overview.entry(sub).or_default().record(got == r.b, &r.u);
                center_trailing_space.record(center.ends_with(' '), &r.u);
            }
            _ => {
                *other_handler.entry(handler.to_string()).or_default() += 1;
            }
        }
    }

    println!("== HTML corpus byte-parity ==");
    println!("input parse errors: {parse_errors}");
    println!();
    println!(
        "intdot           : {:>6}/{:<6} = {:6.2}%   (first fail: {:?})",
        intdot.ok,
        intdot.total,
        intdot.pct(),
        intdot.first_fail
    );
    println!(
        "  name guard fails: {intdot_name_guard_fail}   tail guard fails: {intdot_tail_guard_fail} (both must be 0)"
    );
    let mut ov_ok = 0;
    let mut ov_total = 0;
    for (sub, t) in &overview {
        ov_ok += t.ok;
        ov_total += t.total;
        println!(
            "{sub:<17}: {:>6}/{:<6} = {:6.2}%   (first fail: {:?})",
            t.ok,
            t.total,
            t.pct(),
            t.first_fail
        );
    }
    println!(
        "overview (all)   : {:>6}/{:<6} = {:6.2}%",
        ov_ok,
        ov_total,
        if ov_total == 0 { 0.0 } else { 100.0 * ov_ok as f64 / ov_total as f64 }
    );
    println!(
        "  center-pane trailing-space rule holds: {}/{} = {:.2}%",
        center_trailing_space.ok,
        center_trailing_space.total,
        center_trailing_space.pct()
    );
    if !other_handler.is_empty() {
        println!("\nunexpected handlers (should be none):");
        for (h, n) in &other_handler {
            println!("  {h}: {n}");
        }
    }
    let grand_ok = intdot.ok + ov_ok;
    let grand_total = intdot.total + ov_total;
    println!(
        "\nALL html         : {:>6}/{:<6} = {:6.2}%",
        grand_ok,
        grand_total,
        if grand_total == 0 { 0.0 } else { 100.0 * grand_ok as f64 / grand_total as f64 }
    );
}

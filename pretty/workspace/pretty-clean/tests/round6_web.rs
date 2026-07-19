//! R6 acceptance — the web rendering mode of theory content.
//!
//! For every captured `main/message` / `main/rules` pane across the corpus, this
//! sweep decodes the response envelope, inverts the producers-clean skin
//! (un-postprocess → logical lines → headed blocks), reconstructs each section's
//! MODEL from its body (spans stripped, entities unescaped, parsed
//! layout-insensitively), RE-RENDERS it through the web-mode path, and asserts
//! byte-identity of every section body — so the layout (width 100 / ribbon 67),
//! the `hl_*` span injection and the entity escaping are all re-derived, not
//! copied through. It also reassembles the whole pane through a faithful replica
//! of the producers skin and byte-compares the entire response envelope.
//!
//! Raw envelopes are materialized (scratchpad/r6/raw) from the sanctioned
//! capture corpus; point `R6_RAW` elsewhere to override.

mod common;

use std::fs;
use std::path::PathBuf;

use common::{
    parse_bare_rule, parse_restriction_block, parse_rule_block, parse_signature,
};
use pretty_clean::web;

fn raw_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("R6_RAW").unwrap_or_else(|_| {
            "/home/kamilner/tamarin-cleanroom/pretty/workspace/scratchpad/r6/raw".into()
        }),
    )
}

// ── envelope decoding (strict; mirrors producers-clean) ──────────────────────

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
                        out.push(char::from_u32(cp).unwrap());
                    }
                    other => panic!("bad escape \\{}", other as char),
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

fn decode_envelope(raw: &str) -> (String, String) {
    let rest = raw.strip_prefix("{\"html\":\"").expect("html key");
    let (html, end) = decode_json_string(rest);
    let rest = rest[end..].strip_prefix("\",\"title\":\"").expect("title key");
    let (title, end) = decode_json_string(rest);
    assert_eq!(&rest[end..], "\"}", "envelope close");
    (html, title)
}

// ── skin inversion ───────────────────────────────────────────────────────────

const P_OPEN: &str = "<p class=\"monospace rules\">";

fn unpostprocess(html: &str) -> Vec<String> {
    assert!(html.ends_with("<br/>\n"), "postprocessed fragment");
    let mut lines: Vec<&str> = html.split("<br/>\n").collect();
    assert_eq!(lines.pop(), Some(""));
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

/// (leading_blank, [(heading, body_lines)]).
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
            .unwrap_or_else(|| panic!("expected heading, got {:?}", lines[i]))
            .to_string();
        i += 1;
        let first = lines[i]
            .strip_prefix(P_OPEN)
            .unwrap_or_else(|| panic!("expected <p>, got {:?}", lines[i]));
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
        blocks.push((heading, body));
    }
    (leading_blank, blocks)
}

fn strip_spans(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('<') {
        out.push_str(&rest[..i]);
        let close = rest[i..].find('>').expect("span tag close") + i;
        let tag = &rest[i..=close];
        assert!(
            tag.starts_with("<span ") || tag == "</span>",
            "unexpected tag in body: {tag:?}"
        );
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

fn unescape(s: &str) -> String {
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
            panic!("unknown entity {:?}", &rest[..rest.len().min(8)]);
        };
        out.push(c);
        rest = &rest[len..];
    }
    out.push_str(rest);
    out
}

/// The captured body of a block as plain text (spans stripped, entities
/// unescaped, leading indent spaces preserved).
fn plain(body: &[String]) -> String {
    body.iter()
        .map(|l| unescape(&strip_spans(l)))
        .collect::<Vec<_>>()
        .join("\n")
}

// ── section reconstruction + re-render ───────────────────────────────────────

/// Split a plain-text section into blocks at col-0 lines matching `head`,
/// trimming the blank lines between blocks.
fn split_at_heads(plain: &str, head: &str) -> Vec<String> {
    let lines: Vec<&str> = plain.lines().collect();
    let starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.starts_with(head))
        .map(|(i, _)| i)
        .collect();
    let mut out = Vec::new();
    for (k, &a) in starts.iter().enumerate() {
        let end = starts.get(k + 1).copied().unwrap_or(lines.len());
        let mut b = end;
        while b > a + 1 && lines[b - 1].is_empty() {
            b -= 1;
        }
        out.push(lines[a..b].join("\n"));
    }
    out
}


// ── skin replica (for the whole-response byte check) ─────────────────────────

fn escape_heading(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn postprocess(assembled: &str) -> String {
    let mut out = String::new();
    for line in assembled.split('\n') {
        let text = line.trim_start_matches(' ');
        for _ in 0..line.len() - text.len() {
            out.push_str("&nbsp;");
        }
        out.push_str(text);
        out.push_str("<br/>\n");
    }
    out
}

fn json_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

/// Reassemble a pane the way producers `render_pane` does: `<h2>`/`<p>` block
/// skeleton (empty blocks per their mode), postprocess, `{html,title}` envelope.
fn reassemble(
    title: &str,
    leading_blank: bool,
    blocks: &[(&str, Option<&str>)], // (heading, Some(body) | None = omitted)
) -> String {
    let mut chunks: Vec<String> = Vec::new();
    if leading_blank {
        chunks.push(String::new());
    }
    for (heading, body) in blocks {
        let Some(body) = body else { continue };
        let mut chunk = String::new();
        chunk.push_str("<h2>");
        chunk.push_str(&escape_heading(heading));
        chunk.push_str("</h2>\n");
        chunk.push_str(P_OPEN);
        chunk.push_str(body);
        chunk.push_str("</p>");
        chunks.push(chunk);
    }
    let html = postprocess(&chunks.join("\n"));
    format!(
        "{{\"html\":\"{}\",\"title\":\"{}\"}}",
        json_escape(&html),
        json_escape(title)
    )
}

// ── the sweeps ───────────────────────────────────────────────────────────────

fn raw_files(suffix: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(raw_dir())
        .expect("R6 raw dir (materialize scratchpad/r6/raw)")
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_str().unwrap().ends_with(suffix))
        .collect();
    v.sort();
    v
}

#[test]
fn sweep_count() {
    // SECTION-LEVEL acceptance: every captured pane section (Signature,
    // Construction/Deconstruction rules, MSR rules, restrictions) re-rendered
    // through the web path at the pinned (100, 67) with ESCAPED-width layout is
    // byte-identical to its capture. R6_DBG=<kind> shows the first byte diffs,
    // R6_DBGC=<kind> the first content diffs. Asserts zero byte misses at the end
    // (was 206/367 before the escaped-width law; now 367/367).
    let mut ok = 0usize;
    let mut bad = 0usize;
    let mut kinds: std::collections::BTreeMap<&str, (usize, usize)> = Default::default();
    let dbg = std::env::var("R6_DBG").ok();
    let mut shown = 0;
    let mut content_ok = 0usize;
    let mut content_bad = 0usize;
    let mut span_ok = 0usize;
    let mut span_bad = 0usize;
    let mut check = |kind: &'static str, got: String, cap: String| {
        // Layout-independent checks: token content (spans stripped, unescaped,
        // whitespace-collapsed) and span placement (class + normalized inner).
        if norm_content(&got) == norm_content(&cap) {
            content_ok += 1;
        } else {
            content_bad += 1;
            if std::env::var("R6_DBGC").ok().as_deref() == Some(kind) && shown < 3 {
                shown += 1;
                let g = norm_content(&got);
                let c = norm_content(&cap);
                // first differing region
                let gb = g.as_bytes();
                let cb = c.as_bytes();
                let mut i = 0;
                while i < gb.len().min(cb.len()) && gb[i] == cb[i] {
                    i += 1;
                }
                let lo = i.saturating_sub(30);
                eprintln!(
                    "CDIFF[{kind}]\n  MINE …{}…\n  CAP  …{}…",
                    &g[lo..(i + 40).min(g.len())],
                    &c[lo..(i + 40).min(c.len())]
                );
            }
        }
        if span_seq(&got) == span_seq(&cap) {
            span_ok += 1;
        } else {
            span_bad += 1;
        }
        let e = kinds.entry(kind).or_insert((0, 0));
        if got == cap {
            ok += 1;
            e.0 += 1;
        } else {
            bad += 1;
            e.1 += 1;
            if dbg.as_deref() == Some(kind) && shown < 2 {
                shown += 1;
                let g: Vec<&str> = got.lines().collect();
                let c: Vec<&str> = cap.lines().collect();
                for i in 0..g.len().max(c.len()) {
                    let a = g.get(i).copied().unwrap_or("<none>");
                    let b = c.get(i).copied().unwrap_or("<none>");
                    if a != b {
                        eprintln!("DIFF[{kind}] line {i}\n  MINE: {a}\n  CAP : {b}");
                        break;
                    }
                }
            }
        }
    };
    for path in raw_files("__message.raw") {
        let raw = fs::read_to_string(&path).unwrap();
        let (html, title) = decode_envelope(&raw);
        if title != "Message theory" {
            continue;
        }
        let (_, blocks) = parse_blocks(&unpostprocess(&html));
        for (h, body) in &blocks {
            let cap = body.join("\n");
            let pt = plain(body);
            let got = match h.as_str() {
                "Signature" => web::render_signature_body(&parse_signature(&pt)),
                _ => {
                    let rules: Vec<_> = split_at_heads(&pt, "rule ")
                        .iter()
                        .map(|b| parse_bare_rule(b))
                        .collect();
                    web::render_bare_rules_body(&rules)
                }
            };
            check(if h == "Signature" { "sig" } else { "constr" }, got, cap);
        }
    }
    for path in raw_files("__rules.raw") {
        let raw = fs::read_to_string(&path).unwrap();
        let (html, _title) = decode_envelope(&raw);
        let (_, blocks) = parse_blocks(&unpostprocess(&html));
        for (h, body) in &blocks {
            let cap = body.join("\n");
            let pt = plain(body);
            match h.as_str() {
                "Multiset Rewriting Rules" => {
                    let rules: Vec<_> = split_at_heads(&pt, "rule ")
                        .iter()
                        .map(|b| parse_rule_block(b))
                        .collect();
                    check("msr", web::render_msr_body(&rules), cap);
                }
                "Restrictions of the Set of Traces" => {
                    let rs: Vec<_> = split_at_heads(&pt, "restriction ")
                        .iter()
                        .map(|b| parse_restriction_block(b))
                        .collect();
                    check("restr", web::render_restrictions_body(&rs), cap);
                }
                _ => {}
            }
        }
    }
    eprintln!(
        "SWEEP (100,67)  byteOK={ok} byteBAD={bad}  contentOK={content_ok}/{} spanOK={span_ok}/{}  {kinds:?}",
        content_ok + content_bad,
        span_ok + span_bad,
    );
    assert_eq!(bad, 0, "every pane section must render byte-identical");
    assert_eq!(ok, 367, "expected 367 sections (82 sig + 164 constr + 82 msr + 39 restr)");
}

/// Strip spans, unescape entities, collapse all whitespace runs to one space —
/// the layout-independent token content.
fn norm_content(s: &str) -> String {
    let plain = unescape(&strip_spans(s));
    plain.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The ordered sequence of (class, whitespace-collapsed inner-text) spans.
fn span_seq(s: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = s;
    // spans may straddle lines and nest; walk opening tags and match to the
    // NEXT closing tag at the same depth via a simple stack.
    let mut stack: Vec<(String, usize)> = Vec::new();
    let bytes = s;
    let mut pos = 0;
    let _ = (&mut rest, &bytes);
    while let Some(i) = s[pos..].find('<') {
        let at = pos + i;
        let close = s[at..].find('>').unwrap() + at;
        let tag = &s[at..=close];
        if let Some(cls) = tag
            .strip_prefix("<span class=\"")
            .and_then(|t| t.strip_suffix("\">"))
        {
            stack.push((cls.to_string(), close + 1));
        } else if tag == "</span>" {
            if let Some((cls, start)) = stack.pop() {
                let inner = &s[start..at];
                let inner_plain = unescape(&strip_spans(inner));
                out.push((cls, inner_plain.split_whitespace().collect::<Vec<_>>().join(" ")));
            }
        }
        pos = close + 1;
    }
    out.sort();
    out
}

#[test]
fn signature_pane_sweep() {
    // Every message pane's SIGNATURE body renders byte-identical. Reconstruct the
    // Signature from the captured body, re-render through the web path, compose
    // the WHOLE message response through the skin with the CAPTURED (opaque)
    // Construction/Deconstruction bodies, and assert the response bytes. The
    // signature layout (fsep fills at width 100 / ribbon 67) reproduces exactly;
    // the sep-based rule bodies do NOT — see the layout blocker in BEHAVIOR.md.
    let files = raw_files("__message.raw");
    assert!(files.len() >= 80, "expected ~82 message panes, got {}", files.len());
    let mut n = 0;
    for path in &files {
        let raw = fs::read_to_string(path).unwrap();
        let (html, title) = decode_envelope(&raw);
        assert_eq!(title, "Message theory");
        let (leading, blocks) = parse_blocks(&unpostprocess(&html));
        assert!(!leading);
        let heads: Vec<&str> = blocks.iter().map(|(h, _)| h.as_str()).collect();
        assert_eq!(
            heads,
            ["Signature", "Construction Rules", "Deconstruction Rules"]
        );
        let mut refs: Vec<(&str, Option<String>)> = Vec::new();
        let sig = web::render_signature_body(&parse_signature(&plain(&blocks[0].1)));
        refs.push(("Signature", Some(sig)));
        for (h, body) in &blocks[1..] {
            refs.push((h.as_str(), Some(body.join("\n"))));
        }
        let bref: Vec<(&str, Option<&str>)> =
            refs.iter().map(|(h, b)| (*h, b.as_deref())).collect();
        assert_eq!(
            reassemble(&title, false, &bref),
            raw,
            "signature whole-response mismatch for {path:?}"
        );
        n += 1;
    }
    eprintln!("signature_pane_sweep: {n} message panes, signature bodies byte-identical");
}

#[test]
fn full_message_pane_sweep() {
    // WHOLE message response, byte-identical, with ALL THREE sections
    // (Signature, Construction Rules, Deconstruction Rules) reconstructed from
    // their models and re-rendered through the web path — nothing carried from
    // the capture. Escaped-width layout closes the rule bodies the R6 blocker
    // could not.
    let files = raw_files("__message.raw");
    assert!(files.len() >= 80, "expected ~82 message panes, got {}", files.len());
    let mut n = 0;
    for path in &files {
        let raw = fs::read_to_string(path).unwrap();
        let (html, title) = decode_envelope(&raw);
        assert_eq!(title, "Message theory");
        let (_, blocks) = parse_blocks(&unpostprocess(&html));
        let heads: Vec<&str> = blocks.iter().map(|(h, _)| h.as_str()).collect();
        assert_eq!(heads, ["Signature", "Construction Rules", "Deconstruction Rules"]);
        assert_eq!(
            rebuild_pane(&raw),
            raw,
            "message whole-response mismatch for {path:?}"
        );
        n += 1;
    }
    eprintln!("full_message_pane_sweep: {n} message panes byte-identical (all sections from model)");
}

#[test]
fn full_rules_pane_sweep() {
    // WHOLE rules response, byte-identical. `Multiset Rewriting Rules` and
    // `Restrictions of the Set of Traces` are reconstructed from their models and
    // re-rendered; `Fact Symbols with Injective Instances` is OPAQUE solver input
    // (not modeled — BEHAVIOR.md R6) and carried through unchanged.
    let files = raw_files("__rules.raw");
    assert!(files.len() >= 80, "expected ~82 rules panes, got {}", files.len());
    let mut n = 0;
    let mut restr_panes = 0;
    for path in &files {
        let raw = fs::read_to_string(path).unwrap();
        let (html, _title) = decode_envelope(&raw);
        let (_, blocks) = parse_blocks(&unpostprocess(&html));
        if blocks
            .iter()
            .any(|(h, _)| h == "Restrictions of the Set of Traces")
        {
            restr_panes += 1;
        }
        assert_eq!(
            rebuild_pane(&raw),
            raw,
            "rules whole-response mismatch for {path:?}"
        );
        n += 1;
    }
    eprintln!("full_rules_pane_sweep: {n} rules panes byte-identical ({restr_panes} with restrictions; fact-symbol lines carried opaque)");
}

/// Reconstruct a WHOLE pane response from its captured envelope: parse every
/// section, re-render the modeled ones (signature / rules / restrictions) through
/// the web path, carry opaque ones (fact symbols) verbatim, and reassemble
/// through the skin replica. The result should byte-equal the input envelope.
fn rebuild_pane(raw: &str) -> String {
    let (html, title) = decode_envelope(raw);
    let (leading, blocks) = parse_blocks(&unpostprocess(&html));
    let mut bodies: Vec<(&str, String)> = Vec::new();
    for (h, body) in &blocks {
        let pt = plain(body);
        let rendered = match h.as_str() {
            "Signature" => web::render_signature_body(&parse_signature(&pt)),
            "Construction Rules" | "Deconstruction Rules" => {
                let rules: Vec<_> =
                    split_at_heads(&pt, "rule ").iter().map(|b| parse_bare_rule(b)).collect();
                web::render_bare_rules_body(&rules)
            }
            "Multiset Rewriting Rules" => {
                let rules: Vec<_> =
                    split_at_heads(&pt, "rule ").iter().map(|b| parse_rule_block(b)).collect();
                web::render_msr_body(&rules)
            }
            "Restrictions of the Set of Traces" => {
                let rs: Vec<_> = split_at_heads(&pt, "restriction ")
                    .iter()
                    .map(|b| parse_restriction_block(b))
                    .collect();
                web::render_restrictions_body(&rs)
            }
            // Opaque solver input (fact symbols, macros) — carried verbatim.
            _ => body.join("\n"),
        };
        bodies.push((h.as_str(), rendered));
    }
    let bref: Vec<(&str, Option<&str>)> =
        bodies.iter().map(|(h, b)| (*h, Some(b.as_str()))).collect();
    reassemble(&title, leading, &bref)
}

#[test]
#[ignore] // live replay: needs envelopes freshly probed from the HS server (R6_LIVE_RAW)
fn live_replay() {
    // Replays the whole-pane byte gate against theories that were NEVER captured
    // (envelopes probed live from the sanctioned HS server into R6_LIVE_RAW), so
    // the escaped-width layout is exercised on unseen rule/restriction bodies, not
    // the corpus it was derived from.
    let dir = PathBuf::from(std::env::var("R6_LIVE_RAW").expect("set R6_LIVE_RAW"));
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("R6_LIVE_RAW dir")
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_str().unwrap().ends_with(".raw"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no live envelopes in {dir:?}");
    for path in &files {
        let raw = fs::read_to_string(path).unwrap();
        assert_eq!(rebuild_pane(&raw), raw, "live whole-response mismatch for {path:?}");
        eprintln!("live_replay OK: {}", path.file_name().unwrap().to_str().unwrap());
    }
    eprintln!("live_replay: {} live pane(s) byte-identical", files.len());
}

#[test]
fn signature_mutation_check() {
    // Doctoring a rendered signature span must break the whole-response byte gate.
    let path = raw_files("__message.raw").into_iter().next().unwrap();
    let raw = fs::read_to_string(&path).unwrap();
    let (html, title) = decode_envelope(&raw);
    let (_, blocks) = parse_blocks(&unpostprocess(&html));
    let sig = web::render_signature_body(&parse_signature(&plain(&blocks[0].1)));
    let mut refs: Vec<(&str, Option<String>)> = vec![("Signature", Some(sig.clone()))];
    for (h, body) in &blocks[1..] {
        refs.push((h.as_str(), Some(body.join("\n"))));
    }
    let good: Vec<(&str, Option<&str>)> = refs.iter().map(|(h, b)| (*h, b.as_deref())).collect();
    assert_eq!(reassemble(&title, false, &good), raw, "unmutated must match");

    let mutated = sig.replace("hl_keyword", "hl_BOGUS");
    assert_ne!(mutated, sig, "signature must carry a keyword span");
    refs[0] = ("Signature", Some(mutated));
    let bad: Vec<(&str, Option<&str>)> = refs.iter().map(|(h, b)| (*h, b.as_deref())).collect();
    assert_ne!(
        reassemble(&title, false, &bad),
        raw,
        "mutation must break the gate"
    );
}

#[test]
fn escaped_width_discriminates_c_mult_from_d_exp() {
    // The blocker's "impossible" pair, now forced by the escaped-width law. Both
    // constructs are 66 GLYPHS at nest 3, so a glyph-width engine measures them
    // identically and wraps them together at every (width, ribbon). The captures
    // do not: `c_mult`'s BODY carries a `>` in `]->` (escaped 69) and WRAPS to
    // header + three rows; `d_exp`'s premise GROUP has no escapable chars
    // (escaped 66) and KEEPS its `]` on one line. Escaped-width layout reproduces
    // exactly this split at the single pinned (100, 67).
    use pretty_clean::ast::*;
    let v = |n: &str, i: u64| {
        Term::Var(VarSpec { name: n.into(), idx: i, sort: SortHint::Untagged, typ: None })
    };
    let ku = |a: Vec<Term>| Fact { persistent: true, name: "KU".into(), args: a, annotations: vec![] };
    let kd = |a: Vec<Term>| Fact { persistent: true, name: "KD".into(), args: a, annotations: vec![] };

    // c_mult: [ !KU( x ), !KU( x.1 ) ] --[ !KU( (x*x.1) ) ]-> [ !KU( (x*x.1) ) ]
    let m = Term::BinOp(BinOp::Mult, Box::new(v("x", 0)), Box::new(v("x", 1)));
    let c_mult = Rule {
        name: "c_mult".into(),
        modulo: Some("AC".into()),
        attributes: vec![],
        premises: vec![ku(vec![v("x", 0)]), ku(vec![v("x", 1)])],
        actions: vec![ku(vec![m.clone()])],
        conclusions: vec![ku(vec![m])],
        loop_breakers: vec![],
    };
    let c_out = web::render_rule_bare(&c_mult);
    assert_eq!(
        c_out.lines().count(),
        4,
        "c_mult body (escaped 69) wraps to header + three rows:\n{c_out}"
    );

    // d_exp premise group:
    //   [ !KD( x.5^(x.4*x.6*inv((x.2*x.7))) ), !KU( (x.2*x.3*inv(x.4)) ) ] --> [ ... ]
    // its 66-glyph premise group carries no escapable chars (escaped 66) and
    // stays on one line, so the whole body fits (a `-->` deconstruction body).
    let ex = Term::BinOp(
        BinOp::Exp,
        Box::new(v("x", 5)),
        Box::new(Term::BinOp(
            BinOp::Mult,
            Box::new(Term::BinOp(BinOp::Mult, Box::new(v("x", 4)), Box::new(v("x", 6)))),
            Box::new(Term::App(
                "inv".into(),
                vec![Term::BinOp(BinOp::Mult, Box::new(v("x", 2)), Box::new(v("x", 7)))],
            )),
        )),
    );
    let prem2 = Term::BinOp(
        BinOp::Mult,
        Box::new(Term::BinOp(BinOp::Mult, Box::new(v("x", 2)), Box::new(v("x", 3)))),
        Box::new(Term::App("inv".into(), vec![v("x", 4)])),
    );
    let d_exp = Rule {
        name: "d_exp".into(),
        modulo: Some("AC".into()),
        attributes: vec![],
        premises: vec![kd(vec![ex.clone()]), ku(vec![prem2])],
        actions: vec![],
        conclusions: vec![kd(vec![ex])],
        loop_breakers: vec![],
    };
    let d_out = web::render_rule_bare(&d_exp);
    // The premise group keeps its `]` on the same line as its facts — no bare
    // `]` row — which a wrap would produce.
    assert!(
        !d_out.lines().any(|l| unescape(&strip_spans(l)).trim() == "]"),
        "d_exp premise group (escaped 66) keeps its ] on one line:\n{d_out}"
    );
}

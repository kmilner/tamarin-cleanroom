//! Shared strict inversion machinery for the west-pane / proof-tree /
//! welcome byte-replay tests: un-postprocess a captured pane into logical
//! lines, slice the opaque content out of the strict observed skeleton
//! (every frame byte asserted), and rebuild the producer inputs.
//!
//! Used by tests/r2_west_pane.rs (corpus + live pane sweeps) and
//! tests/r3_proof_tree.rs (live proof-tree replays); different binaries use
//! different subsets.
#![allow(dead_code)]

use producers_clean::model::{
    Content, Highlight, LemmaEntry, NavItem, ProofDisplay, ProofScriptPane, ProofTree, ThyPath,
};
use producers_clean::{parse_path, render_path, render_proof_script};

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
pub fn unescape_entities(s: &str) -> String {
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

pub fn slice_pane(inner: &str) -> (u64, ProofScriptPane) {
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

        // 4d. Proof display: the exact sorry step, or a structured proof tree
        // (only the per-node method text stays opaque).
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
            let mut inv = TreeInv {
                lines: &pl,
                pos: 0,
                idx,
                lemma: &name,
            };
            let root = inv.node(0, &mut Vec::new());
            assert_eq!(inv.pos, pl.len(), "proof display fully consumed");
            // The lemma-header wrapper is exactly the root's status class.
            assert_eq!(
                wrapper.as_deref(),
                class_of(&root.status),
                "header wrapper matches proof root status"
            );
            ProofDisplay::Tree(root)
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

// ---------------------------------------------------------------------------
// Proof-tree inversion (R3)
// ---------------------------------------------------------------------------

/// Status class ↔ [`Highlight`] mapping the observed skeleton uses.
pub fn class_of(h: &Highlight) -> Option<&'static str> {
    match h {
        Highlight::None => None,
        Highlight::Good => Some("hl_good"),
        Highlight::Bad => Some("hl_bad"),
        Highlight::Medium => Some("hl_medium"),
        Highlight::Replayed => Some("hl_superfluous"),
    }
}

pub fn hl_of(cls: Option<&str>) -> Highlight {
    match cls {
        None => Highlight::None,
        Some("hl_good") => Highlight::Good,
        Some("hl_bad") => Highlight::Bad,
        Some("hl_medium") => Highlight::Medium,
        Some("hl_superfluous") => Highlight::Replayed,
        Some(other) => panic!("unknown status class {other:?}"),
    }
}

const KW_BY: &str = "<span class=\"hl_keyword\">by</span> ";

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

/// `Some((status, rest))` when `s` opens with a possibly-wrapped keyword
/// structural line (`case ` tail returned; `next`/`qed` return `""`).
fn keyword_line(s: &str, kw: &str) -> Option<(Option<String>, String)> {
    let bare = format!("<span class=\"hl_keyword\">{kw}</span>");
    if let Some(rest) = s.strip_prefix(&bare) {
        // Bare (status-less) form: `case NAME` / whole-line `next` / `qed`.
        return match rest.strip_prefix(' ') {
            Some(name) => Some((None, name.to_string())),
            None if rest.is_empty() => Some((None, String::new())),
            _ => None,
        };
    }
    let w = s.strip_prefix("<span class=\"")?;
    let (cls, rest) = w.split_once("\">")?;
    let rest = rest.strip_prefix(&bare)?;
    let inner = rest.strip_suffix("</span>")?;
    match inner.strip_prefix(' ') {
        Some(name) => Some((Some(cls.to_string()), name.to_string())),
        None if inner.is_empty() => Some((Some(cls.to_string()), String::new())),
        _ => None,
    }
}

/// One parsed step line: the `by ` prefix's wrapper (outer `Option` = prefix
/// present), the anchor/span status class, remove-step presence, the href for
/// an anchored step, and the opaque method text.
struct StepItem {
    indent: usize,
    by: Option<Option<String>>,
    status: Option<String>,
    anchored: bool,
    removable: bool,
    href: Option<String>,
    method: String,
}

/// Strict recursive-descent inversion of a proof display into a
/// [`ProofTree`], asserting every frame byte, status placement and link
/// target on the way.
struct TreeInv<'a> {
    lines: &'a [String],
    pos: usize,
    idx: u64,
    lemma: &'a str,
}

impl TreeInv<'_> {
    /// Classify the line at `pos` without consuming (case/next/qed/step).
    fn peek(&self) -> Option<(&'static str, usize, Option<String>, String)> {
        let line = self.lines.get(self.pos)?;
        let ind = indent_of(line);
        let rest = &line[ind..];
        for kw in ["case", "next", "qed"] {
            if let Some((st, tail)) = keyword_line(rest, kw) {
                return Some((kw, ind, st, tail));
            }
        }
        Some(("step", ind, None, String::new()))
    }

    /// Parse the step item starting at `pos` (consuming the method text's
    /// continuation lines).
    fn step(&mut self) -> StepItem {
        let line = &self.lines[self.pos];
        let indent = indent_of(line);
        let mut rest = line[indent..].to_string();
        // Optional `by ` prefix, bare or status-wrapped.
        let by = if let Some(r) = rest.strip_prefix(KW_BY) {
            let r = r.to_string();
            rest = r;
            Some(None)
        } else if let Some(w) = rest.strip_prefix("<span class=\"") {
            match w.split_once("\">") {
                Some((cls, r)) if r.starts_with(KW_BY) => {
                    let r = r
                        .strip_prefix(KW_BY)
                        .unwrap()
                        .strip_prefix("</span>")
                        .expect("wrapped by-prefix closes");
                    let cls = cls.to_string();
                    rest = r.to_string();
                    Some(Some(cls))
                }
                _ => None,
            }
        } else {
            None
        };
        if let Some(a) = rest.strip_prefix("<a class=\"internal-link proof-step ") {
            // Anchored step: slice the href, then the method up to `</a>`.
            let (cls, a) = a.split_once("\" href=\"").expect("proof-step href");
            let (href, a) = a.split_once("\">").expect("href close");
            let status = match cls {
                "sorry-step" => None,
                other => Some(other.to_string()),
            };
            let (method, tail) = self.until(a.to_string(), "</a>");
            let removable = !tail.is_empty();
            if removable {
                let t = tail
                    .strip_prefix("<a class=\"internal-link remove-step\" href=\"")
                    .unwrap_or_else(|| panic!("remove-step follows: {tail:?}"));
                let t = t.strip_prefix(href).expect("remove-step href matches");
                assert_eq!(t, "\"></a>", "remove-step anchor shape");
            }
            self.pos += 1;
            StepItem {
                indent,
                by,
                status,
                anchored: true,
                removable,
                href: Some(href.to_string()),
                method,
            }
        } else {
            // Replayed (span-only) step: the method sits in a status span;
            // spans nest, so consume until the OUTER span closes.
            let w = rest
                .strip_prefix("<span class=\"")
                .unwrap_or_else(|| panic!("step line shape: {rest:?}"));
            let (cls, first) = w.split_once("\">").expect("status span open");
            let cls = cls.to_string();
            let mut body = first.to_string();
            let mut depth = 1usize;
            let mut method = String::new();
            let tail: String;
            'span: loop {
                let mut i = 0;
                loop {
                    match (body[i..].find("<span"), body[i..].find("</span>")) {
                        (Some(o), Some(c)) if o < c => {
                            depth += 1;
                            i += o + 5;
                        }
                        (_, Some(c)) => {
                            depth -= 1;
                            if depth == 0 {
                                method.push_str(&body[..i + c]);
                                tail = body[i + c + 7..].to_string();
                                self.pos += 1;
                                break 'span;
                            }
                            i += c + 7;
                        }
                        (Some(o), None) => {
                            depth += 1;
                            i += o + 5;
                        }
                        (None, None) => break,
                    }
                }
                method.push_str(&body);
                method.push('\n');
                self.pos += 1;
                body = self.lines[self.pos].clone();
            }
            // Optional remove-step anchor after the method span.
            let (removable, href) = if tail.is_empty() {
                (false, None)
            } else {
                let t = tail
                    .strip_prefix("<a class=\"internal-link remove-step\" href=\"")
                    .unwrap_or_else(|| panic!("remove-step follows span step: {tail:?}"));
                let (href, t) = t.split_once('"').expect("remove-step href close");
                assert_eq!(t, "></a>", "remove-step anchor shape");
                (true, Some(href.to_string()))
            };
            StepItem {
                indent,
                by,
                status: Some(cls),
                anchored: false,
                removable,
                href,
                method,
            }
        }
    }

    /// Collect (possibly across lines) until `marker`; returns (content,
    /// same-line tail after the marker).
    fn until(&mut self, mut tail: String, marker: &str) -> (String, String) {
        let mut out = String::new();
        loop {
            if let Some(p) = tail.find(marker) {
                out.push_str(&tail[..p]);
                return (out, tail[p + marker.len()..].to_string());
            }
            out.push_str(&tail);
            out.push('\n');
            self.pos += 1;
            tail = self.lines[self.pos].clone();
        }
    }

    /// Parse one node (step + case layout) at `depth`, path `sub`.
    fn node(&mut self, depth: usize, sub: &mut Vec<String>) -> ProofTree {
        let st = self.step();
        assert_eq!(st.indent, depth, "step at its node's indent");
        if st.href.is_some() {
            let want = format!(
                "/thy/trace/{}/main/{}",
                self.idx,
                render_path(&ThyPath::Proof {
                    lemma: self.lemma.to_string(),
                    sub: sub.clone(),
                })
                .join("/")
            );
            assert_eq!(st.href.as_deref(), Some(want.as_str()), "canonical step href");
        }
        if let Some(by_cls) = &st.by {
            assert_eq!(by_cls.as_deref(), st.status.as_deref(), "by wrapper = step status");
        }
        let status = hl_of(st.status.as_deref());
        if !st.anchored {
            assert!(
                matches!(status, Highlight::Replayed),
                "span-only steps carry the replayed status"
            );
        }
        let mut cases: Vec<(String, ProofTree)> = Vec::new();
        match self.peek() {
            Some(("step", ind, _, _)) if ind == depth => {
                // Single unnamed continuation at the same indent.
                assert!(st.by.is_none(), "a closed (by) step has no continuation");
                sub.push("_".to_string());
                let child = self.node(depth, sub);
                sub.pop();
                cases.push((String::new(), child));
            }
            Some(("case", ind, _, _)) if ind == depth + 2 => loop {
                let (kw, ind, case_st, name) = self.peek().unwrap();
                assert_eq!((kw, ind), ("case", depth + 2), "case at parent indent + 2");
                assert!(!name.is_empty() && !name.contains('<'), "plain case name");
                self.pos += 1;
                sub.push(name.clone());
                let child = self.node(depth + 2, sub);
                sub.pop();
                assert_eq!(
                    case_st.as_deref(),
                    class_of(&child.status),
                    "case line carries the child's status"
                );
                cases.push((name, child));
                match self.peek() {
                    Some(("next", i2, nst, _)) if i2 == depth => {
                        assert_eq!(
                            nst.as_deref(),
                            st.status.as_deref(),
                            "next carries the parent's status"
                        );
                        self.pos += 1;
                    }
                    Some(("qed", i2, qst, _)) if i2 == depth => {
                        assert_eq!(
                            qst.as_deref(),
                            st.status.as_deref(),
                            "qed carries the parent's status"
                        );
                        self.pos += 1;
                        break;
                    }
                    other => panic!("expected next/qed at indent {depth}, got {other:?}"),
                }
            },
            _ => {}
        }
        if !cases.is_empty() {
            assert!(st.by.is_none(), "a node with cases has no by prefix");
        }
        // Removability, as observed corpus-wide: every step carries the
        // remove-step anchor (replayed leftovers included) EXCEPT the sorry
        // slots — the status-less steps with no real (non-replayed)
        // continuation.
        match (&status, st.anchored) {
            (Highlight::Replayed, false) => {
                assert!(st.removable, "replayed steps keep the remove affordance")
            }
            (Highlight::None, true) => {
                let has_real_child = cases
                    .iter()
                    .any(|(_, c)| !matches!(c.status, Highlight::Replayed));
                assert_eq!(st.removable, has_real_child, "sorry slots are not removable");
            }
            (_, true) => assert!(st.removable, "anchored non-sorry steps are removable"),
            (st2, false) => panic!("span-only step with status {:?}", class_of(st2)),
        }
        ProofTree {
            method_text: st.method,
            status,
            live: st.removable,
            terminal_marker: cases.is_empty() && st.by.is_none(),
            cases,
        }
    }
}

pub fn replay(inner: &str) -> String {
    let (_idx, pane) = slice_pane(inner);
    // Structural expectations that hold corpus-wide ([S16]): five nav items,
    // message first, both sources views present.
    assert_eq!(pane.items.len(), 5, "five nav items");
    assert_eq!(pane.items[0].target, ThyPath::Message);
    render_proof_script(&pane)
}


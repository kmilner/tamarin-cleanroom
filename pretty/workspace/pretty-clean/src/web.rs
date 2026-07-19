//! R6 — the interactive server's WEB rendering mode of theory content.
//!
//! The `main/message` and `main/rules` pane BODIES render the SAME blocks the
//! batch echo models, but: at width 100 / ribbon 67 (the default HughesPJ
//! `Style`, not the batch 110/73); HTML-entity-escaped; with keyword / operator
//! / comment tokens wrapped in `<span class="hl_…">`; and WITHOUT the batch
//! signature header comment. See workspace/BEHAVIOR.md "Web mode (R6)" for the
//! capture provenance of every rule below.
//!
//! ONE model, TWO render targets. The block-doc builders in `term` /
//! `signature` / `rule` / `formula` / `lemma` emit ALL of their text through the
//! web-aware [`w_text`] / [`w_char`] constructors, and their styled glyphs
//! additionally through the [`hl_kw`] / [`hl_op_char`] / [`hl_op_text`] /
//! [`hl_comment`] / [`hl_wrap`] wrappers here. In batch mode (the default)
//! [`w_text`]/[`w_char`] ARE `doc::text`/`doc::char` and each `hl_*` wrapper is
//! the identity — the R1–R5 output is byte-for-byte unchanged. In web mode
//! (inside [`html_render`]) the `hl_*` wrappers add a zero-width span-marker pair
//! — mirroring the sanctioned `Annotated.HughesPJ` `AnnotStart`/`AnnotEnd`:
//! `sized_text(0, …)` sentinels that flow through best/fits/lay at zero width.
//!
//! **Escaped-width layout.** In web mode every visible token is laid out at its
//! ENTITY-ESCAPED width, not its glyph width: the layout engine measures what the
//! output actually shows after escaping (`<`/`>` = 4 columns, `&` = 5, `"` = 6,
//! `'` = 5, every other visible char = 1). [`w_text`]/[`w_char`] realise this by
//! sizing the token to its escaped length while keeping the raw glyph as content
//! ([`crate::doc::sized_text`] supports that size-vs-content divergence). So
//! wrap/fit decisions for rule bodies, bracket groups, quantifier / formula
//! groups and the pair/AC/application fills all judge fit on the escaped form —
//! equivalently, escape first, then lay out. [`html_render`] renders at (100, 67)
//! and runs one pass that entity-escapes the raw text and expands the sentinels
//! into the spans. The span markers stay zero-width, so they never affect layout.

use std::cell::Cell;

use crate::ast::{AcVariants, Restriction, Rule, Signature};
use crate::doc::{beside_op, char, render_with, sized_text, text, Doc};
use crate::{formula, lemma, rule, signature};

/// Web layout parameters — width 100, ribbon 67 (the default HughesPJ `Style`:
/// `lineLength = 100`, `ribbonsPerLine = 1.5`, `round(100/1.5) = 67`). Combined
/// with the escaped-width sizing of [`w_text`]/[`w_char`], these reproduce the
/// captured wrap thresholds exactly: the signature `fsep` fills (no escapable
/// chars, so escaped width = glyph width) reach absolute 78 = nest 11 + ribbon
/// 67, while a nest-3 rule body wraps once its ESCAPED width exceeds 67 — a
/// content-66 body carrying an arrow (`]->` charges its `>` at 4, escaped 69)
/// wraps, whereas a content-66 bracket group with no escapable chars (escaped
/// 66) keeps its `]`. The same escaped measurement forces the pair/AC/app
/// delimiter drops. Census provenance: QUERIES.log R7 (one-line max escaped 67 /
/// wrapped min 69, zero exceptions).
pub const WEB_WIDTH: isize = 100;
pub const WEB_RIBBON: isize = 67;

// ── span markers (zero-width sentinels; expanded by `escape_and_expand`) ──────
//
// Control chars that never occur in theory content (no C0 controls in the
// corpus), so they survive the entity-escape pass untouched and are then
// expanded into span tags.
const M_OPEN: char = '\u{1}'; //  \u{1}<class>\u{2}  ->  <span class="<class>">
const M_SEP: char = '\u{2}';
const M_CLOSE: char = '\u{3}'; //  \u{3}            ->  </span>

thread_local! {
    static HL: Cell<bool> = const { Cell::new(false) };
}

fn hl_on() -> bool {
    HL.with(|c| c.get())
}

/// RAII: enable web-mode styling for the duration of a doc build, restoring the
/// previous state on drop (so nesting and panics leave the flag consistent).
struct HlGuard(bool);
impl HlGuard {
    fn on() -> Self {
        HlGuard(HL.with(|c| c.replace(true)))
    }
}
impl Drop for HlGuard {
    fn drop(&mut self) {
        HL.with(|c| c.set(self.0));
    }
}

/// Wrap `d` in a zero-width span-marker pair carrying `class`.
fn annot(class: &str, d: Doc) -> Doc {
    let open = format!("{M_OPEN}{class}{M_SEP}");
    let close = M_CLOSE.to_string();
    beside_op(sized_text(0, &open), beside_op(d, sized_text(0, &close)))
}

// ── escaped-width sizing (web mode) ──────────────────────────────────────────

/// The column width a character occupies AFTER the entity escaping applied by
/// [`escape_and_expand`]: the length of its escaped form. Any character that is
/// not escaped keeps width 1 (non-ASCII glyphs `∀ ∃ ⇒ …` are one scalar each and
/// pass raw). This is THE production layout charge.
fn escaped_char_width(c: char) -> usize {
    match c {
        '<' | '>' => 4, // &lt; &gt;
        '&' => 5,       // &amp;
        '"' => 6,       // &quot;
        '\'' => 5,      // &#39;
        _ => 1,
    }
}

thread_local! {
    /// The per-character layout charge, [`escaped_char_width`] in all normal
    /// operation. Indirected through a cell so the acceptance suite alone can
    /// DOCTOR it (e.g. to glyph width) and prove the byte gate genuinely depends
    /// on the escaped charging — see `escaped_charging_is_load_bearing`.
    static CHARGE: Cell<fn(char) -> usize> = Cell::new(escaped_char_width);
}

/// The active layout charge for one character (the escaped width in production).
fn char_charge(c: char) -> usize {
    CHARGE.with(|k| k.get())(c)
}

/// Layout charge for a whole string (sum of per-character charges).
fn escaped_width(s: &str) -> usize {
    s.chars().map(char_charge).sum()
}

/// Web-aware `text`: in web mode the token is sized to its ESCAPED width while
/// its raw glyphs stay as the content (escaped later by [`escape_and_expand`]),
/// so the layout engine measures the escaped form. In batch mode it is exactly
/// `doc::text` — the R1–R5 output is byte-for-byte unchanged.
pub(crate) fn w_text(s: &str) -> Doc {
    if hl_on() {
        sized_text(escaped_width(s), s)
    } else {
        text(s)
    }
}

/// Web-aware `char`: the escaped-width counterpart of `doc::char`.
pub(crate) fn w_char(c: char) -> Doc {
    if hl_on() {
        let mut buf = [0u8; 4];
        sized_text(char_charge(c), c.encode_utf8(&mut buf))
    } else {
        char(c)
    }
}

// ── glyph wrappers (identity in batch mode) ──────────────────────────────────

/// A keyword token (`rule`, `modulo`, `functions:`, …) — `hl_keyword`.
pub(crate) fn hl_kw(s: &str) -> Doc {
    if hl_on() {
        annot("hl_keyword", w_text(s))
    } else {
        text(s)
    }
}

/// An operator glyph given as a single char (`=`, `@`, `.`, `(`, `)`, `<`, …) —
/// `hl_operator`.
pub(crate) fn hl_op_char(c: char) -> Doc {
    if hl_on() {
        annot("hl_operator", w_char(c))
    } else {
        char(c)
    }
}

/// An operator glyph given as text (`--[`, `]->`, `-->`, `∀ `, `⇒`, …) —
/// `hl_operator`. `]->` / `-->` carry a `>` and are laid out at their escaped
/// width (6 columns) in web mode.
pub(crate) fn hl_op_text(s: &str) -> Doc {
    if hl_on() {
        annot("hl_operator", w_text(s))
    } else {
        text(s)
    }
}

/// A comment line/block (`// safety formula`, `/* … */`) — `hl_comment`.
pub(crate) fn hl_comment(s: &str) -> Doc {
    if hl_on() {
        annot("hl_comment", w_text(s))
    } else {
        text(s)
    }
}

/// Wrap an already-built sub-document in a span of `class` (used for the
/// multi-line variants / expanded-formula comment blocks, whose interior keeps
/// its own nested keyword/operator spans).
pub(crate) fn hl_wrap(class: &str, d: Doc) -> Doc {
    if hl_on() {
        annot(class, d)
    } else {
        d
    }
}

// ── the escape + span-expansion pass ─────────────────────────────────────────

/// Entity-escape producer text (`& < > " '`, identical to the producers-clean
/// `escape_text`) AND expand the zero-width span sentinels into `<span
/// class="hl_…">` / `</span>`, in one pass. Leading indent spaces are left raw
/// (the producers postprocess turns them into `&nbsp;`); non-ASCII passes raw.
fn escape_and_expand(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + s.len() / 8 + 16);
    let mut it = s.chars();
    while let Some(c) = it.next() {
        match c {
            M_OPEN => {
                out.push_str("<span class=\"");
                for cc in it.by_ref() {
                    if cc == M_SEP {
                        break;
                    }
                    out.push(cc);
                }
                out.push_str("\">");
            }
            M_CLOSE => out.push_str("</span>"),
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Build a styled doc (web mode ON) and render it to an escaped, span-injected
/// HTML fragment body at web params. The rendered logical lines carry raw
/// leading spaces (postprocessed downstream) and one `\n` per line.
fn html_render(build: impl FnOnce() -> Doc) -> String {
    let _g = HlGuard::on();
    let doc = build();
    escape_and_expand(&render_with(WEB_WIDTH, WEB_RIBBON, &doc))
}

// ── block render entry points ────────────────────────────────────────────────

/// The `Signature` message-pane body: `builtins:` / `functions:` / `equations:`
/// with NO batch header comment.
pub fn render_signature_body(sig: &Signature) -> String {
    html_render(|| signature::web_block_doc(sig))
}

/// One BARE rule (header + body only, no variants comment) — the
/// construction/deconstruction message-pane form.
pub fn render_rule_bare(r: &Rule) -> String {
    html_render(|| rule::core_doc(r))
}

/// One FULL rule block (header + body + blank + variants comment) — the MSR
/// rules-pane form.
pub fn render_rule_block(r: &Rule, variants: Option<&AcVariants>) -> String {
    html_render(|| rule::block_doc(r, variants))
}

/// One restriction block (statement, `// safety formula`, expanded-formula
/// comment) — the rules-pane restrictions form.
pub fn render_restriction(r: &Restriction) -> String {
    html_render(|| lemma::restriction_doc(r))
}

/// One formula in bare (top-level) position at web params — for source views
/// and lemma statements (UNVALIDATED: no formula-only web capture; reuses the
/// restriction-validated operator spans).
pub fn render_formula(f: &crate::ast::Formula) -> String {
    html_render(|| formula::doc(f))
}

// ── section-body assemblers ──────────────────────────────────────────────────

/// A `Construction Rules` / `Deconstruction Rules` message-pane body: BARE
/// rules joined by one blank line.
pub fn render_bare_rules_body(rules: &[Rule]) -> String {
    rules
        .iter()
        .map(render_rule_bare)
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// The `Multiset Rewriting Rules` rules-pane body: FULL rule blocks joined by
/// TWO blank lines after a `modulo AC` rule and ZERO after a `modulo E` rule
/// (corpus: 164 AC→2 / 942 E→0), no trailing.
pub fn render_msr_body(rules: &[(Rule, Option<AcVariants>)]) -> String {
    let mut out = String::new();
    for (i, (r, v)) in rules.iter().enumerate() {
        if i > 0 {
            let prev = &rules[i - 1].0;
            out.push_str(if prev.modulo.as_deref() == Some("AC") {
                "\n\n\n"
            } else {
                "\n"
            });
        }
        out.push_str(&render_rule_block(r, v.as_ref()));
    }
    out
}

/// The `Restrictions of the Set of Traces` rules-pane body: restriction blocks
/// joined by one blank line.
pub fn render_restrictions_body(restrictions: &[Restriction]) -> String {
    restrictions
        .iter()
        .map(render_restriction)
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinOp, Fact, Rule, SortHint, Term, VarSpec};

    /// Temporarily install a doctored per-character charge; returns the previous
    /// one so the caller can restore it.
    fn set_charge(f: fn(char) -> usize) -> fn(char) -> usize {
        CHARGE.with(|k| k.replace(f))
    }

    /// The blocker witness `c_mult`: a bare rule whose body is 66 GLYPHS but 69
    /// ESCAPED columns (the single `>` in `]->` charges 4). Captured as a WRAPPED
    /// three-row body (BP_IBS_4 / DH2_original; census in QUERIES.log R7).
    fn c_mult_rule() -> Rule {
        let v = |n: &str, i: u64| {
            Term::Var(VarSpec { name: n.into(), idx: i, sort: SortHint::Untagged, typ: None })
        };
        let m = Term::BinOp(BinOp::Mult, Box::new(v("x", 0)), Box::new(v("x", 1)));
        let ku = |a: Vec<Term>| Fact {
            persistent: true,
            name: "KU".into(),
            args: a,
            annotations: vec![],
        };
        Rule {
            name: "c_mult".into(),
            modulo: Some("AC".into()),
            attributes: vec![],
            premises: vec![ku(vec![v("x", 0)]), ku(vec![v("x", 1)])],
            actions: vec![ku(vec![m.clone()])],
            conclusions: vec![ku(vec![m])],
            loop_breakers: vec![],
        }
    }

    #[test]
    fn escaped_char_width_matches_the_escaper() {
        // Every charge equals the byte length its char expands to in
        // `escape_and_expand`.
        assert_eq!(escaped_char_width('<'), "&lt;".len());
        assert_eq!(escaped_char_width('>'), "&gt;".len());
        assert_eq!(escaped_char_width('&'), "&amp;".len());
        assert_eq!(escaped_char_width('"'), "&quot;".len());
        assert_eq!(escaped_char_width('\''), "&#39;".len());
        assert_eq!(escaped_char_width('x'), 1);
        assert_eq!(escaped_char_width('\u{2200}'), 1); // ∀ passes raw
    }

    #[test]
    fn escaped_charging_is_load_bearing() {
        // Under the production ESCAPED charge the content-66 body wraps to a
        // header + three rows (4 lines), matching the captures. Doctoring the
        // charge to glyph width (the mutation) drops the escaped 69 to 66 <=
        // ribbon 67, so the body stays on one line (2 lines) — a DIFFERENT byte
        // output. Hence the byte gate genuinely rides on the escaped charging.
        let r = c_mult_rule();
        let real = render_rule_bare(&r);
        assert_eq!(real.lines().count(), 4, "escaped charge wraps the body");

        let prev = set_charge(|_| 1); // doctor: charge every char its glyph width
        let mutant = render_rule_bare(&r);
        set_charge(prev);
        assert_eq!(mutant.lines().count(), 2, "glyph charge keeps the body one line");
        assert_ne!(real, mutant, "doctoring the charge changes the rendered bytes");
    }
}

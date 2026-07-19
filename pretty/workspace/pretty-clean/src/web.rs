//! R6 — the interactive server's WEB rendering mode of theory content.
//!
//! The `main/message` and `main/rules` pane BODIES render the SAME blocks the
//! batch echo models, but: at width 100 / ribbon 67 (the default HughesPJ
//! `Style`, not the batch 110/73); HTML-entity-escaped; with keyword / operator
//! / comment tokens wrapped in `<span class="hl_…">`; and WITHOUT the batch
//! signature header comment. See workspace/BEHAVIOR.md "Web mode (R6)" for the
//! capture provenance of every rule below.
//!
//! ONE model, TWO render targets. The block-doc builders in `signature` /
//! `rule` / `formula` / `lemma` emit their styled glyphs through the [`hl_kw`] /
//! [`hl_op_char`] / [`hl_op_text`] / [`hl_comment`] / [`hl_wrap`] wrappers here.
//! In batch mode (the default) each wrapper is the identity — the R1–R5 output
//! is byte-for-byte unchanged. In web mode (inside [`html_render`]) each wraps
//! its glyph in a zero-width span-marker pair — mirroring the sanctioned
//! `Annotated.HughesPJ` `AnnotStart`/`AnnotEnd`: `sized_text(0, …)` sentinels
//! that flow through best/fits/lay at zero width, so the LAYOUT is identical to
//! the plain render. [`html_render`] then renders at (100, 67) and runs one pass
//! that entity-escapes the text and expands the sentinels into the spans.

use std::cell::Cell;

use crate::ast::{AcVariants, Restriction, Rule, Signature};
use crate::doc::{beside_op, char, render_with, sized_text, text, Doc};
use crate::{formula, lemma, rule, signature};

/// Web layout parameters — width 100, ribbon 67 (the default HughesPJ `Style`:
/// `lineLength = 100`, `ribbonsPerLine = 1.5`, `round(100/1.5) = 67`). These
/// reproduce the SIGNATURE fills byte-for-byte (the fsep continuations reach
/// absolute 78 = nest 11 + ribbon 67, and all 82 message-pane signatures render
/// byte-identical; see tests/round6_web.rs `signature_pane_sweep`).
///
/// KNOWN LAYOUT LIMITATION (BEHAVIOR.md "Web mode (R6)" blocker): a single
/// (width, ribbon) with this faithful HughesPJ engine does NOT reproduce the
/// captured wrap thresholds for the `sep`-based constructs (rule bodies,
/// restriction quantifiers) and the pair/AC delimiter drops — a nest-3 rule
/// body of content 66 (`c_mult`) wraps in the captures while a nest-3
/// bracket-group premise of content 66 (`d_exp`) keeps its `]`, yet this engine
/// measures both identically; and the signature fills demand ribbon 67 with
/// width ≥ 78 while the rule bodies demand an effective width ~67. The span
/// vocabulary, entity escaping, section structure/separators and token content
/// are all reproduced; the residual is line-wrapping only. See the blocker note.
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

// ── glyph wrappers (identity in batch mode) ──────────────────────────────────

/// A keyword token (`rule`, `modulo`, `functions:`, …) — `hl_keyword`.
pub(crate) fn hl_kw(s: &str) -> Doc {
    if hl_on() {
        annot("hl_keyword", text(s))
    } else {
        text(s)
    }
}

/// An operator glyph given as a single char (`=`, `@`, `.`, `(`, `)`, …) —
/// `hl_operator`.
pub(crate) fn hl_op_char(c: char) -> Doc {
    if hl_on() {
        annot("hl_operator", char(c))
    } else {
        char(c)
    }
}

/// An operator glyph given as text (`--[`, `]->`, `-->`, `∀ `, `⇒`, …) —
/// `hl_operator`.
pub(crate) fn hl_op_text(s: &str) -> Doc {
    if hl_on() {
        annot("hl_operator", text(s))
    } else {
        text(s)
    }
}

/// A comment line/block (`// safety formula`, `/* … */`) — `hl_comment`.
pub(crate) fn hl_comment(s: &str) -> Doc {
    if hl_on() {
        annot("hl_comment", text(s))
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

//! R2 — the proof-script WEST pane (the theory index left of every page).
//!
//! The pane is a flat document of logical lines put through the R1 per-line
//! postprocess (each line + `<br/>\n`, leading spaces → `&nbsp;`) with ONE
//! trailing space after the last break (all 478 corpus panes end `<br/>\n `
//! [S16]). Element order (BEHAVIOR.md §12, [S16]):
//!
//!   1. `theory NAME begin` header (NAME links to `main/help`);
//!   2. per nav item: a blank line, then the item link line;
//!   3. blank + the `add lemma` link for position `<first>`;
//!   4. per lemma: blank, declaration line(s), the edit-or-delete link line,
//!      the proof display (a `by sorry` step or the R3-rendered proof-tree
//!      lines, [`crate::prooftree`]), blank, that lemma's `add lemma` link;
//!   5. blank + `end` — with zero lemmas the pane shows TWO blanks here
//!      (steps 3→5 with an empty step 4, observed in the 2 lemma-less panes).
//!
//! Every internal href is `/thy/trace/{idx}/main/` + an R5-rendered theory
//! path — R2 constructs its links through [`crate::path::render`].
//!
//! The lemma DECLARATION is `lemma {name}{attributes}:` (attributes text
//! opaque, possibly multi-line [S17]), then the quantifier/formula block at
//! indent 2 with the inline-vs-vertical layout rule pinned by [S18]/[L14]:
//! a single-line formula joins the quantifier line when the assembled line's
//! ESCAPED width (tags stripped, entities counted at their escaped length)
//! is ≤ 69 characters; otherwise the quantifier stands alone and the formula
//! lines follow, each at the 2-space block indent plus its own relative
//! indent. A proved/disproved lemma's header (declaration + edit/delete line)
//! is wrapped in a single status span (`hl_good` / `hl_bad`); an incomplete
//! proof leaves the header unwrapped [S16].

use crate::html::{escape_text, postprocess_lines};
use crate::model::{Highlight, LemmaEntry, ProofDisplay, ProofScriptPane, ThyPath};
use crate::path;
use crate::prooftree;

/// The inline-layout width limit: a quantifier+formula line inlines iff its
/// escaped width is ≤ this. Bisected live to exactly 69/70 on all four probe
/// families ([L14]); the corpus straddles it at 65/71 ([S18]).
const INLINE_WIDTH_LIMIT: usize = 69;

/// Escaped width of a document line: HTML tags count 0, every other character
/// (entities at their escaped length, unicode per char) counts 1 ([S18][L14]).
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

fn href(idx: u64, target: &ThyPath) -> String {
    format!("/thy/trace/{idx}/main/{}", path::render(target).join("/"))
}

fn add_link(idx: u64, pos: &str) -> String {
    format!(
        r#"<a class="internal-link add" href="{}">add lemma</a>"#,
        href(idx, &ThyPath::Add(pos.to_string()))
    )
}

fn keyword(kw: &str) -> String {
    format!(r#"<span class="hl_keyword">{kw}</span>"#)
}

/// Push one lemma's lines (declaration, edit/delete, proof display).
fn push_lemma(elems: &mut Vec<String>, idx: u64, lemma: &LemmaEntry) {
    let name_html = escape_text(&lemma.name);
    let decl = format!("{} {}{}:", keyword("lemma"), name_html, lemma.attributes);
    let edit_delete = format!(
        r#"<a class="internal-link edit" href="{}">edit lemma</a>  or  <a class="internal-link delete" href="{}">delete lemma</a>"#,
        href(idx, &ThyPath::Edit(lemma.name.clone())),
        href(idx, &ThyPath::Delete(lemma.name.clone()))
    );
    // Header wrapper: one status span around declaration + edit/delete line,
    // carrying the proof ROOT's status class; none for `sorry` or an
    // incomplete (status-less root) proof [S16][S19].
    let header_status = match &lemma.proof {
        ProofDisplay::Tree(root) => match &root.status {
            Highlight::None => None,
            Highlight::Good => Some("hl_good"),
            Highlight::Bad => Some("hl_bad"),
            Highlight::Medium => Some("hl_medium"),
            Highlight::Replayed => Some("hl_superfluous"),
        },
        ProofDisplay::Unproven => None,
    };
    match header_status {
        Some(status) => {
            elems.push(format!(r#"<span class="{status}">{decl}"#));
            elems.push(format!("{edit_delete}</span>"));
        }
        None => {
            elems.push(decl);
            elems.push(edit_delete);
        }
    }
    // The quantifier/formula block goes BETWEEN the declaration and the
    // edit/delete line.
    let at = elems.len() - 1;
    let mut block: Vec<String> = Vec::new();
    if let [only_line] = lemma.formula.lines.as_slice() {
        let candidate = format!("  {} {}", lemma.quantifier, only_line);
        if escaped_width(&candidate) <= INLINE_WIDTH_LIMIT {
            block.push(candidate);
        }
    }
    if block.is_empty() {
        block.push(format!("  {}", lemma.quantifier));
        for line in &lemma.formula.lines {
            block.push(format!("  {line}"));
        }
    }
    elems.splice(at..at, block);
    // Proof display.
    match &lemma.proof {
        ProofDisplay::Unproven => elems.push(format!(
            r#"{} <a class="internal-link proof-step sorry-step" href="{}">{}</a>"#,
            keyword("by"),
            href(idx, &ThyPath::Proof { lemma: lemma.name.clone(), sub: vec![] }),
            keyword("sorry")
        )),
        ProofDisplay::Tree(root) => {
            elems.extend(prooftree::render_tree_lines(idx, &lemma.name, root))
        }
    }
}

/// Render the inner HTML of the west proof-script pane (the content of the
/// page's proof-script container, final trailing space included).
pub fn render_index(pane: &ProofScriptPane) -> String {
    let idx = pane.index;
    let mut elems: Vec<String> = Vec::new();
    elems.push(format!(
        r#"{} <a class="internal-link help" href="{}">{}</a> {}"#,
        keyword("theory"),
        href(idx, &ThyPath::Help),
        escape_text(&pane.theory_name),
        keyword("begin")
    ));
    for item in &pane.items {
        elems.push(String::new());
        elems.push(format!(
            r#"<a class="internal-link" href="{}"><strong>{}</strong> {}</a>"#,
            href(idx, &item.target),
            item.label,
            item.annotation
        ));
    }
    elems.push(String::new());
    elems.push(add_link(idx, "<first>"));
    if pane.lemmas.is_empty() {
        // A lemma-less pane leaves TWO blank lines before `end` (both
        // lemma-less corpus panes [S16]).
        elems.push(String::new());
    }
    for lemma in &pane.lemmas {
        elems.push(String::new());
        push_lemma(&mut elems, idx, lemma);
        elems.push(String::new());
        elems.push(add_link(idx, &lemma.name));
    }
    elems.push(String::new());
    elems.push(keyword("end"));
    let mut out = postprocess_lines(&elems.join("\n"));
    out.push(' ');
    out
}

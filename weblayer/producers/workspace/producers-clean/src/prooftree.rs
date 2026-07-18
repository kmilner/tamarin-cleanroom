//! R3 — proof-tree + proof-method HTML.
//!
//! Lays a pre-computed proof tree out as the logical lines embedded in the
//! west proof-script pane (the pane applies the shared per-line postprocess).
//! The tree SHAPE, per-node METHOD TEXT (pre-rendered HTML, possibly
//! multi-line) and per-node status are opaque inputs; this module owns the
//! line grammar, indentation, links and status-class attachment, all pinned
//! from the 478-pane capture corpus + live probes (BEHAVIOR.md §16–§18,
//! QUERIES.log [S19]–[S21], [L16]–[L18]).
//!
//! The grammar, per node at indent `d` with URL path `P` (= `proof/{lemma}`
//! plus one segment per case on the root-to-node walk, `_` for an unnamed
//! continuation):
//!
//!   * step line: `d` spaces + [`by ` prefix] + the step itself.
//!     - the `by ` prefix (keyword span + trailing space, wrapped in the
//!       node's status span) appears iff the node has NO cases and is not a
//!       terminal MARKER (`SOLVED`-style lines carry no `by`) [S19];
//!     - a Replayed-status node (a proof-script leftover) has NO proof-step
//!       link: the method is wrapped in the `hl_superfluous` status span
//!       [S20]; every other node's step is `<a class="internal-link
//!       proof-step CLS" href="/thy/trace/{idx}/main/P">METHOD</a>` where CLS
//!       is the status class (`sorry-step` for a status-less node);
//!     - a LIVE node's step ends with `<a class="internal-link remove-step"
//!       href="…same…"></a>`; the only non-live steps observed are the
//!       `sorry` slots (leaf, or carrying a replayed continuation) — replayed
//!       leftovers keep the remove affordance [S19][S20].
//!   * a single unnamed case continues at the SAME indent, path `P/_`,
//!     with no case/next/qed framing;
//!   * named cases: each case opens `case NAME` at `d+2` (wrapped in the
//!     CHILD's status span) followed by the child at `d+2`; siblings are
//!     separated by `next` at `d`, and the block closes with `qed` at `d` —
//!     `next`/`qed` carry the PARENT node's status span [S19][S21].
//!
//! Status→class: Good→`hl_good`, Bad→`hl_bad`, Medium→`hl_medium` (never
//! observed — assumed by pattern, see BEHAVIOR.md §18), Replayed→
//! `hl_superfluous`; None renders bare (no wrapping span) [S19][S20].
//!
//! Out of scope (solver-computed, other panes): the constraint-system pane,
//! the applicable-proof-methods listing, and the method text itself.

use crate::html::escape_text;
use crate::model::{Highlight, ProofTree, ThyPath};
use crate::path;

/// The status span class each highlight attaches, or None for bare rendering.
fn status_class(status: &Highlight) -> Option<&'static str> {
    match status {
        Highlight::None => None,
        Highlight::Good => Some("hl_good"),
        Highlight::Bad => Some("hl_bad"),
        Highlight::Medium => Some("hl_medium"),
        Highlight::Replayed => Some("hl_superfluous"),
    }
}

/// Wrap `inner` in the node-status span (`<span class="CLS">…</span>`), or
/// return it unchanged for a status-less node.
fn wrap_status(status: &Highlight, inner: &str) -> String {
    match status_class(status) {
        Some(cls) => format!(r#"<span class="{cls}">{inner}</span>"#),
        None => inner.to_string(),
    }
}

fn keyword(kw: &str) -> String {
    format!(r#"<span class="hl_keyword">{kw}</span>"#)
}

/// Render a lemma's proof tree as the logical lines embedded in the west
/// pane (each element one document line; multi-line method texts keep their
/// embedded newlines). The pane joins these with the rest of the document
/// and applies the shared postprocess.
pub fn render_tree_lines(index: u64, lemma: &str, root: &ProofTree) -> Vec<String> {
    let mut lines = Vec::new();
    render_node(&mut lines, index, lemma, &mut Vec::new(), root, 0);
    lines
}

/// Render a lemma's proof tree as one pre-postprocess document fragment
/// (the [`render_tree_lines`] elements joined with newlines).
pub fn render_tree(index: u64, lemma: &str, root: &ProofTree) -> String {
    render_tree_lines(index, lemma, root).join("\n")
}

fn indent(n: usize) -> String {
    " ".repeat(n)
}

/// One node: its step line, then its case layout (see module docs).
fn render_node(
    lines: &mut Vec<String>,
    index: u64,
    lemma: &str,
    sub: &mut Vec<String>,
    node: &ProofTree,
    depth: usize,
) {
    let ind = indent(depth);
    let href = format!(
        "/thy/trace/{index}/main/{}",
        path::render(&ThyPath::Proof {
            lemma: lemma.to_string(),
            sub: sub.clone(),
        })
        .join("/")
    );
    let mut step = ind.clone();
    // `by ` prefix: a case-less node that is a real method, not a terminal
    // marker ([S19]: contradiction / zero-case solve / sorry take `by`;
    // SOLVED does not).
    if node.cases.is_empty() && !node.terminal_marker {
        step.push_str(&wrap_status(&node.status, &format!("{} ", keyword("by"))));
    }
    if matches!(node.status, Highlight::Replayed) {
        // Replayed leftover: no proof-step link (nothing computed to show),
        // the method sits in the status span instead [S20].
        step.push_str(&wrap_status(&node.status, &node.method_text));
    } else {
        let cls = status_class(&node.status).unwrap_or("sorry-step");
        step.push_str(&format!(
            r#"<a class="internal-link proof-step {cls}" href="{href}">{}</a>"#,
            node.method_text
        ));
    }
    // Remove-step affordance: every step except the `sorry` slots — replayed
    // leftovers included (they remain addressable and removable) [S19][S20].
    if node.live {
        step.push_str(&format!(
            r#"<a class="internal-link remove-step" href="{href}"></a>"#
        ));
    }
    lines.push(step);

    match node.cases.as_slice() {
        [] => {}
        [(name, child)] if name.is_empty() => {
            // Single unnamed continuation: same indent, path segment `_`.
            sub.push("_".to_string());
            render_node(lines, index, lemma, sub, child, depth);
            sub.pop();
        }
        cases => {
            let case_ind = indent(depth + 2);
            for (i, (name, child)) in cases.iter().enumerate() {
                if i > 0 {
                    lines.push(format!(
                        "{ind}{}",
                        wrap_status(&node.status, &keyword("next"))
                    ));
                }
                lines.push(format!(
                    "{case_ind}{}",
                    wrap_status(
                        &child.status,
                        &format!("{} {}", keyword("case"), escape_text(name))
                    )
                ));
                sub.push(name.clone());
                render_node(lines, index, lemma, sub, child, depth + 2);
                sub.pop();
            }
            lines.push(format!(
                "{ind}{}",
                wrap_status(&node.status, &keyword("qed"))
            ));
        }
    }
}

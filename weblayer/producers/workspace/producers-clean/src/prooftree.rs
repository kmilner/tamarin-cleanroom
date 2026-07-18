//! R3 — proof-tree + proof-method HTML.
//!
//! Lay out a pre-computed proof tree as nested HTML: per node a proof-step link
//! carrying the opaque method text and (unless it is the unproven leaf) a
//! remove-step affordance; named child cases each nest one level deeper with a
//! `case NAME` header; sibling cases separated by `next`; a block closed by
//! `qed`. Structural keywords carry the node's highlight status. The METHOD
//! TEXT and the tree SHAPE are pre-computed inputs — this module only maps the
//! tree to indented, linked HTML.
//!
//! Out of scope (solver-computed, not this cluster): the constraint-system
//! pane, the applicable-proof-methods listing, and the method text itself.
//!
//! Observe the proved `overview` west panes and the `main/proof` targets for
//! the indentation, the by/case/next/qed grammar, and the link path encoding.

use crate::model::ProofTree;

/// Render a lemma's proof tree as the nested HTML embedded in the west pane.
pub fn render_tree(_index: u64, _lemma: &str, _root: &ProofTree) -> String {
    // TODO(sealed): R3. Recurse the tree; emit step/case/next/qed lines at the
    // observed indentation with proof-step + remove-step links.
    unimplemented!("R3: proof-tree HTML")
}

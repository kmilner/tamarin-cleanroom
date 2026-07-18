//! HughesPJ `Doc` layout engine (width-aware pretty-printing).
//!
//! OUT OF THE GPL-ERASURE SURFACE. This layout algebra is BSD-licensed (Haskell
//! `pretty-1.1.3.6`) and has ALREADY been clean-roomed by the graphdot cluster:
//! `../../../graphdot/workspace/graph-clean/src/pretty.rs`, derived from
//! `../../../graphdot/sanctioned/pretty-1.1.3.6`.
//!
//! Do NOT re-derive `best`/`fits`/`nicest` from the GPL side. REUSE the
//! graphdot engine — copy it in here (BSD terms permit) or add a path
//! dependency. Theory-echo layout parameters: line width = 110, ribbon = 73.
//! The tamarin-specific part (WHICH combinators, in what nesting, with what
//! literal strings) is what R1–R4 build on top of this.

// TODO(sealed): vendor the graphdot BSD Doc engine here and expose the
// combinators the renderers use (`<>`, `<+>`, `$$`, `$+$`, sep, cat, fsep,
// fcat, nest, text, render_with(width, ribbon)).

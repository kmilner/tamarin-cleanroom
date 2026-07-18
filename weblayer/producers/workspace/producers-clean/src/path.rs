//! R5 — the theory-path grammar (URL <-> structured path).
//!
//! Parse the percent-decoded wildcard segment after `/thy/trace/<idx>/<handler>/`
//! into a structured path, and render a structured path back to the URL
//! segments the fragments link to. A pure, HTML-free grammar; the residual
//! shared with the (already clean-roomed) dispatch route parser, kept here
//! because the producers construct these links directly.
//!
//! Observe: every link href in every fragment target is a rendered path; probe
//! the live oracle for the parse side (which segment shapes 404 vs resolve) and
//! for the escaping / underscore-prefix quirks.

use crate::model::ThyPath;

/// Parse a raw wildcard path into a structured path (None if unparseable).
pub fn parse(_raw: &str) -> Option<ThyPath> {
    // TODO(sealed): R5. Percent-decode + match the handler grammar.
    unimplemented!("R5: parse theory path")
}

/// Render a structured path to its URL segments.
pub fn render(_path: &ThyPath) -> Vec<String> {
    // TODO(sealed): R5. Inverse of parse, incl. the observed escaping quirks.
    unimplemented!("R5: render theory path")
}

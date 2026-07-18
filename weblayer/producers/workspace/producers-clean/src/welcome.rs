//! R4 — the welcome / index page (`/`) and housekeeping fragments.
//!
//! The index page: a fixed frame (core-team paragraph, license/intro text, the
//! upload form) around a loaded-theory table (one row per version) and an
//! optional one-shot banner (upload succeeded / failed). Plus the small fixed
//! bodies: the static help block, and the housekeeping responses
//! (robots / cancel-ack / the invalid-args page). Almost fully producer-owned;
//! the only opaque inputs are the per-row name / time / origin strings.
//!
//! Observe the `/` captures (index page), the *_help.html targets (the static
//! help block + the env line), and the housekeeping route bodies.

use crate::model::Welcome;

/// Render the index (`/`) page body.
pub fn render_welcome(_w: &Welcome) -> String {
    // TODO(sealed): R4. Frame + theory-table rows + banner, verbatim per the
    // `/` captures.
    unimplemented!("R4: welcome/index page")
}

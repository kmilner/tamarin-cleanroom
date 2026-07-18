//! R4 — the welcome / index page (`/`) and housekeeping bodies.
//!
//! The index page is a fixed frame (head + loading bar + "Running Tamarin
//! VERSION" header + logo + credits/license block + upload form) around three
//! slots: an optional one-shot flash paragraph right after `<body>`, the
//! version text, and the loaded-theory table rows. Everything else is
//! byte-constant — the frame segments are verbatim captured output
//! (compatibility content), including the doubled `</script></script>`
//! closers and the row cell's unclosed `<em>` (BEHAVIOR.md §19, QUERIES.log
//! [L19]–[L21]). No trailing newline.
//!
//! One row per loaded theory version, index ascending:
//!
//! ```text
//! <tr><td><a href="/thy/trace/IDX/overview/help">NAME</a></td>
//!     <td>TIME</td><td>Original | <em>Modified</td><td>ORIGIN</td></tr>
//! ```
//!
//! NAME/TIME/ORIGIN are opaque display strings, entity-escaped here (the
//! metachar-filename upload probe [L21] pins the escaping to the shared R1
//! set). ORIGIN is the load path for a disk theory and the bare uploaded
//! filename for an upload; a proof/upload-derived version shows the
//! `<em>Modified` cell, the initial load `Original`.
//!
//! Flash paragraph (`<p class="message">…</p>`, first child of `<body>`)
//! [L20]: upload success `Loaded new theory!`; a POST without a usable file
//! `Post request failed.`; a load error the pre-computed message text
//! (entity-escaped, embedded newlines preserved).
//!
//! Housekeeping bodies [L19]: `robots.txt` and the cancel acknowledgement
//! are fixed plain-text bodies without a trailing newline; a missing static
//! file answers a fixed plain-text body; the Invalid-Arguments page wraps
//! `<li>` message items in the observed error shell.

use crate::html::escape_text;
use crate::model::{Banner, TheoryRow, Welcome};

const PRE_FLASH: &str = include_str!("welcome_pre_flash.html");
const PRE_VERSION: &str = include_str!("welcome_pre_version.html");
const PRE_ROWS: &str = include_str!("welcome_pre_rows.html");
const POST_ROWS: &str = include_str!("welcome_post_rows.html");

const INVALID_ARGS_PRE: &str = include_str!("invalid_args_pre.html");
const INVALID_ARGS_POST: &str = include_str!("invalid_args_post.html");

/// `GET /robots.txt` body (no trailing newline) [L19].
pub const ROBOTS_BODY: &str = "User-agent: *";

/// `GET /kill?path=…` acknowledgement body (no trailing newline) [L19].
pub const CANCEL_ACK_BODY: &str = "Canceled request!";

/// Missing `/static/**` file body [L19].
pub const FILE_NOT_FOUND_BODY: &str = "File not found";

/// The one-shot flash paragraph after `<body>`, or empty for a plain GET.
fn flash(banner: &Banner) -> String {
    let msg = match banner {
        Banner::None => return String::new(),
        Banner::Loaded => "Loaded new theory!".to_string(),
        Banner::Failed => "Post request failed.".to_string(),
        Banner::Custom(m) => escape_text(m),
    };
    format!("<p class=\"message\">{msg}</p>")
}

fn row(r: &TheoryRow) -> String {
    format!(
        "<tr><td><a href=\"/thy/trace/{}/overview/help\">{}</a></td><td>{}</td><td>{}</td><td>{}</td></tr>",
        r.index,
        escape_text(&r.name),
        escape_text(&r.time),
        if r.modified { "<em>Modified" } else { "Original" },
        escape_text(&r.origin),
    )
}

/// Render the index (`/`) page body.
pub fn render_welcome(w: &Welcome) -> String {
    let mut out = String::with_capacity(4096 + 256 * w.rows.len());
    out.push_str(PRE_FLASH);
    out.push_str(&flash(&w.banner));
    out.push_str(PRE_VERSION);
    out.push_str(&escape_text(&w.version));
    out.push_str(PRE_ROWS);
    for r in &w.rows {
        out.push_str(&row(r));
    }
    out.push_str(POST_ROWS);
    out
}

/// Render the 400 Invalid-Arguments page around the given message items
/// (observed instance: `GET /kill` without a path) [L19].
pub fn render_invalid_args(messages: &[String]) -> String {
    let mut out = String::with_capacity(INVALID_ARGS_PRE.len() + INVALID_ARGS_POST.len() + 64);
    out.push_str(INVALID_ARGS_PRE);
    out.push_str("<ul>");
    for m in messages {
        out.push_str("<li>");
        out.push_str(&escape_text(m));
        out.push_str("</li>\n");
    }
    out.push_str("</ul>\n");
    out.push_str(INVALID_ARGS_POST);
    out
}

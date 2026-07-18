//! R1 — theory-view CENTER section fragments.
//!
//! The `main/message`, `main/rules`, `main/tactic` panes share one skeleton
//! (BEHAVIOR.md §4–7): a plain-text document of headed blocks —
//! `<h2>HEADING</h2>` on its own line, then a `<p class="monospace rules">`
//! paragraph glued to the first body line and closed on the last — put through
//! the per-line postprocess and wrapped in the `{html,title}` envelope. Blocks
//! follow each other with no blank line between; empty bodies render per the
//! block's [`EmptyRender`] mode.
//!
//! `main/help` (BEHAVIOR.md §8) is a different, single-line template rendered
//! by [`render_help_pane`]: the env line + the fixed static help block, with
//! NO per-line postprocess.

use crate::html::{escape_text, html_envelope, postprocess_lines};
use crate::model::{ContentPane, EmptyRender, HelpPane};

/// The fixed help block shown under the env line of every `main/help`
/// response, byte-identical across all 81 corpus captures [S09]. Verbatim
/// captured output (compatibility content), including the stray `</span>`
/// after the Tamarin span.
pub const HELP_STATIC_HTML: &str = include_str!("help_static.html");

/// Render a center-section content pane to its response-body bytes (the
/// `{html,title}` envelope). The entry point for `main/message`, `main/rules`
/// and `main/tactic`.
pub fn render_pane(pane: &ContentPane) -> String {
    let mut chunks: Vec<String> = Vec::new();
    for block in &pane.blocks {
        if block.body.is_empty() {
            match block.when_empty {
                EmptyRender::Omit => continue,
                EmptyRender::BlankLine => {
                    chunks.push(String::new());
                    continue;
                }
                EmptyRender::Keep => {}
            }
        }
        let mut chunk = String::new();
        chunk.push_str("<h2>");
        chunk.push_str(&escape_text(&block.heading));
        chunk.push_str("</h2>\n<p class=\"monospace rules\">");
        chunk.push_str(&block.body.lines.join("\n"));
        chunk.push_str("</p>");
        chunks.push(chunk);
    }
    html_envelope(&pane.title, &postprocess_lines(&chunks.join("\n")))
}

/// Render the `main/help` pane to its response-body bytes.
///
/// Shape (BEHAVIOR.md §8): `<p>Theory: NAME (Loaded at TIME from ORIGIN)
/// BANNER</p>` + the static help block; title `Theory: NAME`. Single line —
/// no postprocess, no trailing newline. An empty banner leaves the observed
/// `) </p>` bytes; a non-empty banner is raw HTML passthrough.
pub fn render_help_pane(help: &HelpPane) -> String {
    let mut html = String::with_capacity(HELP_STATIC_HTML.len() + 256);
    html.push_str("<p>Theory: ");
    html.push_str(&escape_text(&help.theory_name));
    html.push_str(" (Loaded at ");
    html.push_str(&escape_text(&help.load_time));
    html.push_str(" from ");
    html.push_str(&escape_text(&help.origin));
    html.push_str(") ");
    html.push_str(&help.wf_banner_html);
    html.push_str("</p>");
    html.push_str(HELP_STATIC_HTML);
    let mut title = String::with_capacity(8 + help.theory_name.len());
    title.push_str("Theory: ");
    title.push_str(&help.theory_name);
    html_envelope(&title, &html)
}

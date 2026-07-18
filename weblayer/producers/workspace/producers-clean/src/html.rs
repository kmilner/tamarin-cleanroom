//! R1 core — the shared HTML fragment skin every producer reuses.
//!
//! Three transforms, each pinned by observation (see workspace/BEHAVIOR.md §2–3
//! and QUERIES.log [S07]–[S12], [L03]–[L06]):
//!   * entity-escaping of producer-owned text;
//!   * the per-line postprocess: every logical line becomes its text (leading
//!     spaces as `&nbsp;` runs) followed by `<br/>` + newline;
//!   * the three JSON response envelopes: `{html,title}`, `{redirect}`,
//!     `{alert}` — compact, `html` key first, minimal string escaping.

/// Entity-escape a run of producer-owned text.
///
/// The full escape set, forced through the help env line via a metachar
/// filename [L06] and matching the corpus-wide entity inventory [S10]:
/// `&`→`&amp;`, `<`→`&lt;`, `>`→`&gt;`, `"`→`&quot;`, `'`→`&#39;`.
/// Everything else (backslash included) passes through unchanged.
pub fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
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

/// Apply the per-line postprocess to an assembled multi-line document.
///
/// Every newline-separated logical line — first, last, and empty ones alike —
/// is emitted as its text followed by `<br/>` and a real newline; a line's
/// LEADING run of spaces becomes one `&nbsp;` per space (interior runs are
/// left as spaces) [S03][S10]. An empty line is exactly `<br/>\n`. Tabs never
/// occur in observed fragments [S10] and pass through unchanged.
pub fn postprocess_lines(assembled: &str) -> String {
    let mut out = String::with_capacity(assembled.len() + assembled.len() / 3 + 8);
    for line in assembled.split('\n') {
        let text = line.trim_start_matches(' ');
        for _ in 0..line.len() - text.len() {
            out.push_str("&nbsp;");
        }
        out.push_str(text);
        out.push_str("<br/>\n");
    }
    out
}

/// JSON string escaping as the observed envelopes use it: `"`→`\"`, `\`→`\\`,
/// newline→`\n`, tab→`\t` — the complete corpus escape inventory [S09]. Other
/// C0 controls never occur; they get the standard short forms / `\u00XX`
/// (documented arbitrary choice, BEHAVIOR.md §2). Non-ASCII stays raw UTF-8.
fn json_escape_into(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
}

/// Serialize an html fragment + pane title into the `{html,title}` envelope:
/// compact JSON, `html` key first [S07][S08][S12].
pub fn html_envelope(title: &str, html: &str) -> String {
    let mut out = String::with_capacity(html.len() + title.len() + 24);
    out.push_str("{\"html\":\"");
    json_escape_into(&mut out, html);
    out.push_str("\",\"title\":\"");
    json_escape_into(&mut out, title);
    out.push_str("\"}");
    out
}

/// The `{redirect}` envelope: `{"redirect":"<url>"}` (1157 corpus instances
/// [S07]).
pub fn redirect_envelope(url: &str) -> String {
    let mut out = String::with_capacity(url.len() + 16);
    out.push_str("{\"redirect\":\"");
    json_escape_into(&mut out, url);
    out.push_str("\"}");
    out
}

/// The `{alert}` envelope: `{"alert":"<msg>"}` (forced live via `del/path/help`
/// [L04]).
pub fn alert_envelope(msg: &str) -> String {
    let mut out = String::with_capacity(msg.len() + 13);
    out.push_str("{\"alert\":\"");
    json_escape_into(&mut out, msg);
    out.push_str("\"}");
    out
}

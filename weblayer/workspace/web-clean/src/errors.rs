//! Minimal status pages: Not Found (404), Invalid Arguments (400), and Method
//! Not Supported (405).
//!
//! Observed by live probing: requests that fail routing or argument validation
//! return a full HTML page built from the standard `<head>` link set and the
//! standard tail, with the loading bar, a status `<h1>`, and a small body. The
//! three pages differ only in the `<title>`/`<h1>` and the body fragment:
//!
//! * **404** (unmatched route / unknown theory index): `<h1>Not Found</h1>` and a
//!   `<p>` echoing the HTML-escaped request path.
//! * **400** (a matched route missing a required query argument, e.g. `GET /kill`
//!   with no `path`): `<h1>Invalid Arguments</h1>` and a `<ul>` of messages.
//! * **405** (a matched route reached with an unsupported HTTP method, e.g. `POST
//!   /kill`): `<h1>Method Not Supported</h1>` and `Method <code>M</code> not
//!   supported`.

use crate::escape::html_escape;
use crate::shell_template::{SIMPLE_HEAD_A, SIMPLE_HEAD_B, SIMPLE_TAIL};

/// Build a status page with the given `<title>` and body fragment.
fn simple_page(title: &str, body: &str) -> String {
    let mut out = String::with_capacity(
        SIMPLE_HEAD_A.len() + title.len() + SIMPLE_HEAD_B.len() + body.len() + SIMPLE_TAIL.len(),
    );
    out.push_str(SIMPLE_HEAD_A);
    out.push_str(title);
    out.push_str(SIMPLE_HEAD_B);
    out.push_str(body);
    out.push_str(SIMPLE_TAIL);
    out
}

/// The 404 Not Found page. The request path is HTML-escaped before being echoed.
pub fn render_not_found(request_path: &str) -> String {
    let body = format!("<h1>Not Found</h1>\n<p>{}</p>\n", html_escape(request_path));
    simple_page("Not Found", &body)
}

/// The 400 Invalid Arguments page listing one `<li>` per message (each escaped).
pub fn render_invalid_args(messages: &[&str]) -> String {
    let mut items = String::new();
    for m in messages {
        items.push_str("<li>");
        items.push_str(&html_escape(m));
        items.push_str("</li>\n");
    }
    let body = format!("<h1>Invalid Arguments</h1>\n<ul>{items}</ul>\n");
    simple_page("Invalid Arguments", &body)
}

/// The 405 Method Not Supported page naming the offending HTTP method.
pub fn render_bad_method(method: &str) -> String {
    let body = format!("<h1>Method Not Supported</h1>\n<p>Method <code>{method}</code> not supported</p>\n");
    simple_page("Method Not Supported", &body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_echoes_path() {
        let p = render_not_found("/thy/trace/1/main/nope");
        assert!(p.contains("<title>Not Found</title>"));
        assert!(p.contains("<h1>Not Found</h1>\n<p>/thy/trace/1/main/nope</p>"));
        assert!(p.ends_with("</body></html>"));
    }

    #[test]
    fn invalid_args_lists_messages() {
        let p = render_invalid_args(&["No path to kill specified!"]);
        assert!(p.contains("<title>Invalid Arguments</title>"));
        assert!(p.contains("<h1>Invalid Arguments</h1>\n<ul><li>No path to kill specified!</li>\n</ul>\n"));
    }

    #[test]
    fn bad_method_names_method() {
        let p = render_bad_method("POST");
        assert!(p.contains("<title>Method Not Supported</title>"));
        assert!(p.contains("<p>Method <code>POST</code> not supported</p>"));
    }
}

//! Static-asset serving and the small fixed text bodies.
//!
//! `/static/**` is served by a filesystem handler distinct from the dynamic
//! routes: the content type is derived from the file extension (no `charset`),
//! there are no caching headers, and a missing file yields a plain-text `404`
//! with the body `File not found` (no full HTML page). The mapping and the
//! missing-file body are reproduced here; the file bytes themselves are on-disk
//! assets supplied by the environment.
//!
//! The other fixed bodies: `robots.txt`, the `/kill` cancellation confirmation,
//! and the `/favicon.ico` redirect target.

/// Body returned by the static handler for a path that does not resolve to a
/// file (`text/plain`, no trailing newline).
pub const STATIC_NOT_FOUND: &str = "File not found";

/// `robots.txt` body (`text/plain`, no trailing newline).
pub const ROBOTS_TXT: &str = "User-agent: *";

/// `/kill?path=…` confirmation body (`text/plain`, no trailing newline).
pub const KILL_CANCELED: &str = "Canceled request!";

/// The message listed on the `400 Invalid Arguments` page when `/kill` is hit
/// without a `path` query argument.
pub const KILL_NO_PATH_MSG: &str = "No path to kill specified!";

/// Location a `GET /favicon.ico` redirects to (`303`).
pub const FAVICON_TARGET: &str = "/static/img/favicon.ico";

/// Content type served for a static asset, chosen by the last path segment's
/// extension. A file with no recognised extension (e.g. `LICENSE`) is served as
/// `application/octet-stream`. Note: static content types carry **no** `charset`.
pub fn static_content_type(path: &str) -> &'static str {
    let name = path.rsplit('/').next().unwrap_or(path);
    // Longest-suffix first so compound extensions resolve before their tail.
    let ext = match name.rfind('.') {
        Some(dot) => &name[dot + 1..],
        None => "",
    };
    match ext {
        "css" => "text/css",
        "js" => "application/javascript",
        "png" => "image/png",
        "ico" => "image/vnd.microsoft.icon",
        "gif" => "image/gif",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "html" | "htm" => "text/html",
        "txt" => "text/plain",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_types_by_extension() {
        assert_eq!(static_content_type("css/tamarin-prover-ui.css"), "text/css");
        assert_eq!(static_content_type("js/jquery.js"), "application/javascript");
        assert_eq!(static_content_type("img/tamarin-logo-3-0-0.png"), "image/png");
        assert_eq!(static_content_type("img/favicon.ico"), "image/vnd.microsoft.icon");
        // No extension -> octet-stream (as observed for /static/LICENSE).
        assert_eq!(static_content_type("LICENSE"), "application/octet-stream");
    }
}

//! Version banner and the maude readiness preamble.
//!
//! `--version`/`-V` writes to two streams (see `workspace/BEHAVIOR.md` §10): the
//! banner (tamarin line + warranty block + `Generated from:` block) goes to
//! stdout, and the maude readiness preamble goes to stderr. They only appeared
//! interleaved under the merged oracle. The stdout banner is stored as a template
//! with `{{SLOT}}` markers for the build/runtime metadata; the static text is
//! byte-exact observed output.

use crate::stream::Streams;

const VERSION_TEMPLATE: &str = include_str!("../fixtures/version.tmpl");

/// Maude tool identity used in the readiness preamble.
#[derive(Debug, Clone)]
pub struct MaudeInfo {
    /// Tool path as printed inside the quotes (default `maude`).
    pub path: String,
    /// Reported maude version, e.g. `3.5.1`.
    pub version: String,
}

impl Default for MaudeInfo {
    fn default() -> Self {
        MaudeInfo { path: "maude".to_string(), version: "3.5.1".to_string() }
    }
}

/// Build/runtime metadata that fills the version banner slots.
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub maude: MaudeInfo,
    pub tamarin_version: String,
    pub copyright_years: String,
    /// Everything after `Git revision: ` (revision + dirty note + branch).
    pub git_description: String,
    /// The `Compiled at:` timestamp text.
    pub compiled_at: String,
}

/// The stdout version banner, filling every slot in the observed template.
pub fn render_version(info: &VersionInfo) -> String {
    VERSION_TEMPLATE
        .replace("{{TAMARIN_VERSION}}", &info.tamarin_version)
        .replace("{{COPYRIGHT_YEARS}}", &info.copyright_years)
        .replace("{{MAUDE_VERSION}}", &info.maude.version)
        .replace("{{GIT_DESCRIPTION}}", &info.git_description)
        .replace("{{COMPILED_AT}}", &info.compiled_at)
}

/// The full `--version`/`-V` output across both streams: the banner on stdout and
/// the maude readiness preamble on stderr.
pub fn frame_version(info: &VersionInfo) -> Streams {
    Streams { out: render_version(info), err: maude_preamble(&info.maude) }
}

/// The 3-line maude readiness preamble emitted before batch/variants processing
/// (each line terminated by a newline).
pub fn maude_preamble(maude: &MaudeInfo) -> String {
    format!(
        "maude tool: '{}'\n checking version: {}. OK.\n checking installation: OK.\n",
        maude.path, maude.version
    )
}

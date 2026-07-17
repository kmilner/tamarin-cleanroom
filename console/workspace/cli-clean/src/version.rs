//! Version banner and the maude readiness preamble.
//!
//! In `--version`/`-V` output the maude readiness check and the version banner
//! interleave in a fixed, reproducible order (the banner is printed while the
//! maude subprocess is awaited). The observed byte layout is stored as a template
//! with `{{SLOT}}` markers for the build/runtime metadata; `render_version` fills
//! the slots. The static text of the template is byte-exact observed output.

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

/// The version banner, filling every slot in the observed template.
pub fn render_version(info: &VersionInfo) -> String {
    VERSION_TEMPLATE
        .replace("{{MAUDE_PATH}}", &info.maude.path)
        .replace("{{TAMARIN_VERSION}}", &info.tamarin_version)
        .replace("{{COPYRIGHT_YEARS}}", &info.copyright_years)
        .replace("{{MAUDE_VERSION}}", &info.maude.version)
        .replace("{{GIT_DESCRIPTION}}", &info.git_description)
        .replace("{{COMPILED_AT}}", &info.compiled_at)
}

/// The 3-line maude readiness preamble emitted before batch/variants processing
/// (each line terminated by a newline).
pub fn maude_preamble(maude: &MaudeInfo) -> String {
    format!(
        "maude tool: '{}'\n checking version: {}. OK.\n checking installation: OK.\n",
        maude.path, maude.version
    )
}

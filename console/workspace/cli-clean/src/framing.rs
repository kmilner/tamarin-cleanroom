//! Batch-mode output framing around an opaque theory payload.
//!
//! The theory pretty-print itself (`theory <name> … end`, including any
//! `WARNING:` block and the `Generated from:` comment) is treated as an opaque
//! payload slot. This module builds everything AROUND it: the maude preamble, the
//! per-theory progress lines, and the "summary of summaries" block.

use crate::version::{maude_preamble, MaudeInfo};

/// Width of the `=` rule that brackets the summary block (observed: 78).
const RULE_WIDTH: usize = 78;

/// Per-theory progress phases, in the order printed for a fully closed theory.
#[derive(Debug, Clone, Copy)]
pub enum Phase {
    Loaded,
    Translated,
    DerivationChecksStarted,
    DerivationChecksEnded,
    Closed,
}

impl Phase {
    fn text(self) -> &'static str {
        match self {
            Phase::Loaded => "Theory loaded",
            Phase::Translated => "Theory translated",
            Phase::DerivationChecksStarted => "Derivation checks started",
            Phase::DerivationChecksEnded => "Derivation checks ended",
            Phase::Closed => "Theory closed",
        }
    }
}

/// The five progress phases a closed theory reports.
pub const CLOSED_PHASES: [Phase; 5] = [
    Phase::Loaded,
    Phase::Translated,
    Phase::DerivationChecksStarted,
    Phase::DerivationChecksEnded,
    Phase::Closed,
];

/// One progress line: `[Theory <name>] <phase>\n`.
pub fn progress_line(theory: &str, phase: Phase) -> String {
    format!("[Theory {}] {}\n", theory, phase.text())
}

fn progress_block(theory: &str, phases: &[Phase]) -> String {
    phases.iter().map(|p| progress_line(theory, *p)).collect()
}

/// Whether a lemma quantifies over all traces or asserts an existing trace.
#[derive(Debug, Clone, Copy)]
pub enum TraceKind {
    AllTraces,
    ExistsTrace,
}

impl TraceKind {
    fn text(self) -> &'static str {
        match self {
            TraceKind::AllTraces => "all-traces",
            TraceKind::ExistsTrace => "exists-trace",
        }
    }
}

/// Per-lemma verdict.
#[derive(Debug, Clone, Copy)]
pub enum LemmaResult {
    Verified,
    Falsified,
    AnalysisIncomplete,
}

impl LemmaResult {
    fn text(self) -> &'static str {
        match self {
            LemmaResult::Verified => "verified",
            LemmaResult::Falsified => "falsified",
            LemmaResult::AnalysisIncomplete => "analysis incomplete",
        }
    }
}

/// One line of the summary body.
#[derive(Debug, Clone)]
pub enum SummaryEntry {
    Lemma { name: String, kind: TraceKind, result: LemmaResult, steps: u64 },
    /// `WARNING: <count> wellformedness check failed!` (note the singular "check").
    Warning { count: u64 },
}

impl SummaryEntry {
    fn text(&self) -> String {
        match self {
            SummaryEntry::Lemma { name, kind, result, steps } => {
                format!("{} ({}): {} ({} steps)", name, kind.text(), result.text(), steps)
            }
            SummaryEntry::Warning { count } => {
                format!("WARNING: {} wellformedness check failed!", count)
            }
        }
    }
}

/// The "summary of summaries" block, bracketed by `=` rules. `processing_time`
/// is rendered as seconds with two decimals (e.g. `0.39s`).
pub fn render_summary(analyzed_path: &str, processing_time: f64, entries: &[SummaryEntry]) -> String {
    let rule = "=".repeat(RULE_WIDTH);
    let mut out = String::new();
    out.push_str(&rule);
    out.push('\n');
    out.push_str("summary of summaries:\n");
    out.push('\n');
    out.push_str(&format!("analyzed: {}\n", analyzed_path));
    out.push('\n');
    out.push_str(&format!("  processing time: {:.2}s\n", processing_time));
    out.push_str("  \n"); // literal two-space separator line
    for e in entries {
        out.push_str(&format!("  {}\n", e.text()));
    }
    out.push('\n');
    out.push_str(&rule);
    out.push('\n');
    out
}

/// The framing that precedes a closed theory's payload: the maude preamble
/// followed by the five progress lines.
pub fn frame_prefix(maude: &MaudeInfo, theory: &str) -> String {
    let mut out = maude_preamble(maude);
    out.push_str(&progress_block(theory, &CLOSED_PHASES));
    out
}

/// Full framing for a processed (closed) theory: maude preamble, the five
/// progress lines, the opaque `payload`, then the summary block. `payload` must
/// be the pretty-printed theory ending with `end\n`.
pub fn frame_processed(
    maude: &MaudeInfo,
    theory: &str,
    payload: &str,
    analyzed_path: &str,
    processing_time: f64,
    entries: &[SummaryEntry],
) -> String {
    let mut out = frame_prefix(maude, theory);
    out.push_str(payload);
    out.push('\n'); // blank line between payload and summary rule
    out.push_str(&render_summary(analyzed_path, processing_time, entries));
    out
}

/// Framing for `--parse-only`: a single `Theory loaded` progress line then the
/// opaque payload. No maude preamble and no summary block.
pub fn frame_parse_only(theory: &str, payload: &str) -> String {
    let mut out = String::new();
    out.push_str(&progress_line(theory, Phase::Loaded));
    out.push_str(payload);
    out
}

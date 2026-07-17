//! Stream-aware output framing around opaque theory payloads.
//!
//! A batch run writes to two independent streams (see `workspace/BEHAVIOR.md`
//! §10). This module models which text lands on which stream and assembles both
//! byte-exactly. The theory pretty-print (`theory <name> … end`, including any
//! `WARNING:` block and the `Generated from:` comment) is an opaque payload slot;
//! everything around it — the maude preamble, the per-theory progress markers, and
//! the "summary of summaries" block — is built here.
//!
//! Stream assignment: the maude preamble and progress markers go to stderr;
//! payloads and the summary block go to stdout.

use crate::stream::Streams;
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

/// One progress line: `[Theory <name>] <phase>\n` (stderr).
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

/// One line of a theory's summary body.
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

/// One theory's contribution to the "summary of summaries" block.
#[derive(Debug, Clone)]
pub struct Summary {
    /// The `analyzed:` path, echoed exactly as given on the command line.
    pub analyzed: String,
    /// The `output:` path, present only when the theory was written to a file
    /// (`-o`/`-O`).
    pub output: Option<String>,
    /// Rendered as seconds with two decimals (e.g. `0.39s`).
    pub processing_time: f64,
    pub entries: Vec<SummaryEntry>,
}

impl Summary {
    /// The theory's block: the `analyzed:` header, a blank line, the aligned
    /// key/value rows (`output:` when present, then `processing time:`), the
    /// two-space separator line, then one line per entry.
    fn render_block(&self) -> String {
        let mut b = String::new();
        b.push_str(&format!("analyzed: {}\n", self.analyzed));
        b.push('\n');

        // Aligned key/value rows: labels right-padded to the widest label, then a
        // single space, so every value starts at the same column.
        let mut rows: Vec<(&str, String)> = Vec::new();
        if let Some(o) = &self.output {
            rows.push(("output:", o.clone()));
        }
        rows.push(("processing time:", format!("{:.2}s", self.processing_time)));
        let width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
        for (label, value) in &rows {
            b.push_str("  ");
            b.push_str(label);
            for _ in 0..(width - label.len()) {
                b.push(' ');
            }
            b.push(' ');
            b.push_str(value);
            b.push('\n');
        }

        b.push_str("  \n"); // literal two-space separator line
        for e in &self.entries {
            b.push_str(&format!("  {}\n", e.text()));
        }
        b
    }
}

/// The full "summary of summaries" section as it appears on stdout: a leading
/// blank line (the payload/summary separator, always present), the top `=` rule,
/// the header, each theory block preceded by a blank line, a trailing blank line,
/// and the bottom rule.
pub fn render_summary(summaries: &[Summary]) -> String {
    let rule = "=".repeat(RULE_WIDTH);
    let mut out = String::new();
    out.push('\n');
    out.push_str(&rule);
    out.push('\n');
    out.push_str("summary of summaries:\n");
    for s in summaries {
        out.push('\n');
        out.push_str(&s.render_block());
    }
    out.push('\n');
    out.push_str(&rule);
    out.push('\n');
    out
}

/// A theory processed by a closed batch run.
#[derive(Debug, Clone)]
pub struct BatchTheory {
    /// Name used in the `[Theory <name>]` progress markers (stderr).
    pub name: String,
    /// The printed theory payload (stdout), or `None` when it was written to an
    /// output file instead.
    pub payload: Option<String>,
    /// Further progress lines the engine emitted for this theory after the five
    /// closed phases (stderr), verbatim and newline-terminated. Empty for a plain
    /// analyze run; under `--prove` it carries the `[Saturating Sources] …` lines.
    pub extra_progress: String,
    /// This theory's summary contribution (stdout).
    pub summary: Summary,
}

/// A closed batch run over one or more theories.
///
/// stderr: the maude preamble, then per theory the five progress markers followed
/// by any extra engine progress. stdout: the printed payloads (in order), then one
/// combined summary block.
pub fn frame_batch(maude: &MaudeInfo, theories: &[BatchTheory]) -> Streams {
    let mut s = Streams::default();
    s.err.push_str(&maude_preamble(maude));
    for t in theories {
        s.err.push_str(&progress_block(&t.name, &CLOSED_PHASES));
        s.err.push_str(&t.extra_progress);
    }
    for t in theories {
        if let Some(p) = &t.payload {
            s.out.push_str(p);
        }
    }
    let summaries: Vec<Summary> = theories.iter().map(|t| t.summary.clone()).collect();
    s.out.push_str(&render_summary(&summaries));
    s
}

/// A theory under `--parse-only`: its name (for the single progress marker) and
/// its opaque payload.
#[derive(Debug, Clone)]
pub struct LoadedTheory {
    pub name: String,
    pub payload: String,
}

/// A `--parse-only` run: stderr carries one `Theory loaded` marker per theory (no
/// preamble); stdout carries the concatenated payloads (no summary block).
pub fn frame_parse_only(theories: &[LoadedTheory]) -> Streams {
    let mut s = Streams::default();
    for t in theories {
        s.err.push_str(&progress_line(&t.name, Phase::Loaded));
        s.out.push_str(&t.payload);
    }
    s
}

/// A `variants` run: stdout carries the variant payload; stderr carries only the
/// maude preamble (no progress markers, no summary).
pub fn frame_variants(maude: &MaudeInfo, payload: &str) -> Streams {
    Streams { out: payload.to_string(), err: maude_preamble(maude) }
}

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

/// Per-lemma verdict. The printed text of a falsified verdict depends on the
/// lemma's [`TraceKind`] (see [`verdict_phrase`]); the verified, incomplete, and
/// cannot-be-finished verdicts print the same for both kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LemmaResult {
    /// The proof closed: `verified`.
    Verified,
    /// A trace was found (all-traces) / no trace exists (exists-trace); the
    /// printed phrase is kind-dependent (see [`verdict_phrase`]).
    Falsified,
    /// The proof was not driven to a terminal state — a lemma not selected for
    /// proving, a `--prove`/`--lemma` pattern that matched nothing, or a proof cut
    /// short by `--bound`: `analysis incomplete`. The `(<N> steps)` count is the
    /// explored depth, and a bounded cutoff adds no other marker.
    AnalysisIncomplete,
    /// The proof reached a terminal `UNFINISHABLE` step and could not continue:
    /// `analysis cannot be finished (reducible operators in subterms)`. Observed
    /// only for reducible operators appearing inside subterm goals; renders
    /// uniformly for both trace-kinds (no falsified-style suffix) and composes with
    /// the `--diff` side prefixes exactly like any other verdict. Distinct from
    /// [`AnalysisIncomplete`](Self::AnalysisIncomplete): a `--bound` cutoff of the
    /// same lemma prints `analysis incomplete`, not this phrase.
    AnalysisCannotBeFinished,
}

/// Which system a summary line describes. A non-diff run reports every lemma as
/// [`LemmaSide::Whole`] (no prefix). Under `--diff` the prover verifies each
/// ordinary lemma separately against the two projected systems, prefixing its
/// line with `RHS`/`LHS`, and reports the auto-generated observational-equivalence
/// lemma as a [`LemmaSide::Diff`] line (`DiffLemma:` prefix, no trace-kind). In a
/// diff summary the entries appear in the order: for each ordinary lemma its
/// `RHS` line then its `LHS` line, and finally the single `Diff` line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LemmaSide {
    Whole,
    Rhs,
    Lhs,
    Diff,
}

/// The verdict phrase for a `<result>` under a given [`TraceKind`]. `verified`,
/// `analysis incomplete`, and `analysis cannot be finished (reducible operators in
/// subterms)` read the same for both kinds; a `falsified` verdict carries a
/// kind-dependent suffix: `- found trace` for an all-traces lemma (a
/// counter-example trace was produced) and `- no trace found` for an exists-trace
/// lemma (no witnessing trace exists).
fn verdict_phrase(result: LemmaResult, kind: TraceKind) -> &'static str {
    match result {
        LemmaResult::Verified => "verified",
        LemmaResult::AnalysisIncomplete => "analysis incomplete",
        LemmaResult::AnalysisCannotBeFinished => {
            "analysis cannot be finished (reducible operators in subterms)"
        }
        LemmaResult::Falsified => match kind {
            TraceKind::AllTraces => "falsified - found trace",
            TraceKind::ExistsTrace => "falsified - no trace found",
        },
    }
}

/// One lemma's verdict line in a theory's summary body. In a non-diff run this is
/// `<name> (<kind>): <verdict> (<steps> steps)`; under `--diff` a projected
/// ordinary lemma prefixes that with `RHS :  `/`LHS :  `, and the
/// observational-equivalence lemma renders as
/// `DiffLemma:  <name> : <verdict> (<steps> steps)` (no trace-kind).
#[derive(Debug, Clone)]
pub struct LemmaOutcome {
    pub name: String,
    /// The lemma's trace quantification. Ignored when [`side`](Self::side) is
    /// [`LemmaSide::Diff`]: an observational-equivalence lemma has no trace-kind,
    /// and its `falsified` verdict always reads `- found trace`.
    pub kind: TraceKind,
    pub result: LemmaResult,
    /// The `<N>` in `(<N> steps)` — the number of proof steps explored (also the
    /// value shown for bounded/incomplete analyses, reflecting the explored depth).
    pub steps: u64,
    /// Which projected system this line reports (see [`LemmaSide`]).
    pub side: LemmaSide,
}

impl LemmaOutcome {
    /// A non-diff lemma line (`<name> (<kind>): <verdict> (<N> steps)`).
    pub fn whole(
        name: impl Into<String>,
        kind: TraceKind,
        result: LemmaResult,
        steps: u64,
    ) -> Self {
        LemmaOutcome { name: name.into(), kind, result, steps, side: LemmaSide::Whole }
    }

    /// A `--diff` projected ordinary-lemma line for one side ([`LemmaSide::Rhs`]
    /// or [`LemmaSide::Lhs`]).
    pub fn projected(
        side: LemmaSide,
        name: impl Into<String>,
        kind: TraceKind,
        result: LemmaResult,
        steps: u64,
    ) -> Self {
        LemmaOutcome { name: name.into(), kind, result, steps, side }
    }

    /// A `--diff` observational-equivalence line
    /// (`DiffLemma:  <name> : <verdict> (<N> steps)`).
    pub fn diff_lemma(name: impl Into<String>, result: LemmaResult, steps: u64) -> Self {
        LemmaOutcome {
            name: name.into(),
            kind: TraceKind::AllTraces,
            result,
            steps,
            side: LemmaSide::Diff,
        }
    }

    /// The single result line, without its `  ` body indent.
    fn line(&self) -> String {
        match self.side {
            LemmaSide::Diff => format!(
                "DiffLemma:  {} : {} ({} steps)",
                self.name,
                verdict_phrase(self.result, TraceKind::AllTraces),
                self.steps
            ),
            LemmaSide::Whole | LemmaSide::Rhs | LemmaSide::Lhs => {
                let core = format!(
                    "{} ({}): {} ({} steps)",
                    self.name,
                    self.kind.text(),
                    verdict_phrase(self.result, self.kind),
                    self.steps
                );
                match self.side {
                    LemmaSide::Rhs => format!("RHS :  {core}"),
                    LemmaSide::Lhs => format!("LHS :  {core}"),
                    _ => core,
                }
            }
        }
    }
}

/// Prefix shared by the `WARNING:` line and its aligned advisory continuation.
const WARNING_PREFIX: &str = "  WARNING: ";

/// The wellformedness-warning heading of a theory's summary body. Present only
/// when at least one wellformedness check failed.
#[derive(Debug, Clone)]
pub struct WarningSummary {
    /// The `<N>` in `WARNING: <N> wellformedness check failed!` (the noun stays
    /// singular "check" regardless of `<N>`).
    pub failed_checks: u64,
    /// Whether the advisory line `The analysis results might be wrong!` follows
    /// the count line. Observed: emitted exactly when the run is a proving run
    /// (`--prove`), independent of how many lemmas were actually proved (a
    /// proving run with zero matched lemmas still emits it; a plain analyze run
    /// never does).
    pub analysis_maybe_wrong: bool,
}

impl WarningSummary {
    /// The warning heading (one or two lines, each `\n`-terminated). The advisory
    /// continuation is left-padded to align under the text after `WARNING: `.
    fn render(&self) -> String {
        let mut s = String::new();
        s.push_str(WARNING_PREFIX);
        s.push_str(&format!("{} wellformedness check failed!\n", self.failed_checks));
        if self.analysis_maybe_wrong {
            for _ in 0..WARNING_PREFIX.len() {
                s.push(' ');
            }
            s.push_str("The analysis results might be wrong!\n");
        }
        s
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
    /// The wellformedness-warning heading, when any check failed.
    pub warnings: Option<WarningSummary>,
    /// The per-lemma verdict lines, in report order.
    pub lemmas: Vec<LemmaOutcome>,
}

impl Summary {
    /// The theory's block: the `analyzed:` header, a blank line, the aligned
    /// key/value rows (`output:` when present, then `processing time:`), then the
    /// body. The body opens with a `  ` (two-space) line and carries up to two
    /// sections — the warning heading and the lemma lines, in that order — each
    /// section preceded by a `  ` line (so a `  ` line also separates the two when
    /// both are present).
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

        // Body: an opening `  ` (two-space) line, then the present sections joined
        // by another `  ` line — the warning heading first, then the lemma lines.
        b.push_str("  \n");
        let mut sections: Vec<String> = Vec::new();
        if let Some(w) = &self.warnings {
            sections.push(w.render());
        }
        if !self.lemmas.is_empty() {
            let mut lines = String::new();
            for l in &self.lemmas {
                lines.push_str("  ");
                lines.push_str(&l.line());
                lines.push('\n');
            }
            sections.push(lines);
        }
        b.push_str(&sections.join("  \n"));
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

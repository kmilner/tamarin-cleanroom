//! Incremental (streaming) emission of the batch output.
//!
//! The assembled model in [`crate::framing`] builds both complete [`Streams`]
//! and hands them back at the end. A consumer that wants output to appear *as
//! the prover runs* — progress markers and payloads flushed while later theories
//! are still being processed — needs to drive emission event-by-event instead.
//!
//! This module provides that drive surface. A [`Sink`] is the consumer's byte
//! destination, tagged by [`Stream`]; the consumer decides when (or whether) to
//! flush after each call. [`BatchEmitter`] defines *what* bytes are produced and
//! *in what per-stream order* — nothing here forces buffering or a final barrier
//! except the "summary of summaries" block, which is inherently a single trailing
//! block over every theory and so is buffered until [`BatchEmitter::finish`].
//!
//! The per-stream byte total is identical to the assembled model: both paths
//! emit, on stderr, the maude preamble then each theory's progress in order, and,
//! on stdout, each printed payload in order then the one combined summary. The two
//! streams interleave differently in wall-clock time (that is the point of
//! streaming), but each stream's byte sequence is unchanged. The equivalence is
//! exercised by the crate's tests via [`drive_batch`].

use crate::framing::{progress_line, render_summary, BatchTheory, Phase, Summary, CLOSED_PHASES};
use crate::stream::{Stream, Streams};
use crate::version::{maude_preamble, MaudeInfo};

/// A byte destination the incremental emitter writes to, tagged by which standard
/// stream the bytes belong on. Implementors choose flush timing: a
/// flush-per-call sink yields true line-by-line streaming; a buffering sink
/// coalesces. The emitter never assumes either.
pub trait Sink {
    /// Hand `text` to the destination for `stream`. Called once per emitted chunk
    /// in emission order.
    fn emit(&mut self, stream: Stream, text: &str);
}

/// A [`Sink`] that accumulates into an in-memory [`Streams`]. Useful for tests
/// and for callers that want the fully assembled result from the streaming path.
#[derive(Debug, Clone, Default)]
pub struct StreamCollector {
    pub streams: Streams,
}

impl Sink for StreamCollector {
    fn emit(&mut self, stream: Stream, text: &str) {
        self.streams.push(stream, text);
    }
}

/// Drives incremental emission of a closed batch run into a [`Sink`].
///
/// Lifecycle: [`begin`](Self::begin) emits the preamble; then, per theory as the
/// engine produces it, call [`progress`](Self::progress) for each phase reached,
/// [`extra_progress`](Self::extra_progress) for any engine lines after the last
/// phase, [`payload`](Self::payload) when the theory's pretty-print is ready, and
/// [`record_summary`](Self::record_summary) once its verdict is known; finally
/// [`finish`](Self::finish) emits the combined summary. The consumer may flush its
/// sink at any point between calls.
pub struct BatchEmitter<'a, S: Sink> {
    sink: &'a mut S,
    summaries: Vec<Summary>,
}

impl<'a, S: Sink> BatchEmitter<'a, S> {
    /// Start a batch run: emit the three-line maude readiness preamble on stderr.
    pub fn begin(sink: &'a mut S, maude: &MaudeInfo) -> Self {
        sink.emit(Stream::Err, &maude_preamble(maude));
        BatchEmitter { sink, summaries: Vec::new() }
    }

    /// Emit one `[Theory <name>] <phase>` progress marker on stderr.
    pub fn progress(&mut self, theory: &str, phase: Phase) {
        self.sink.emit(Stream::Err, &progress_line(theory, phase));
    }

    /// Emit all five closed-theory progress markers for `theory`, in order.
    pub fn closed_phases(&mut self, theory: &str) {
        for phase in CLOSED_PHASES {
            self.progress(theory, phase);
        }
    }

    /// Emit further engine progress lines on stderr, verbatim (already
    /// newline-terminated), e.g. the `[Saturating Sources] …` lines a `--prove`
    /// run appends after a theory's `Theory closed` marker. A no-op for `""`.
    pub fn extra_progress(&mut self, text: &str) {
        if !text.is_empty() {
            self.sink.emit(Stream::Err, text);
        }
    }

    /// Emit a theory's pretty-printed payload on stdout. Pass `None` when the
    /// theory was written to an output file instead (nothing is emitted).
    pub fn payload(&mut self, payload: Option<&str>) {
        if let Some(p) = payload {
            self.sink.emit(Stream::Out, p);
        }
    }

    /// Record this theory's contribution to the trailing summary. Buffered until
    /// [`finish`](Self::finish), because the "summary of summaries" is one block
    /// over every theory.
    pub fn record_summary(&mut self, summary: Summary) {
        self.summaries.push(summary);
    }

    /// Emit the combined "summary of summaries" block on stdout, closing the run.
    pub fn finish(self) {
        self.sink.emit(Stream::Out, &render_summary(&self.summaries));
    }
}

/// Convenience driver: stream a whole `&[BatchTheory]` into `sink` in the natural
/// per-theory order (preamble; then, per theory, its closed phases, extra
/// progress, payload, and recorded summary; then the combined summary). Produces
/// per-stream bytes identical to [`crate::framing::frame_batch`] on the same
/// inputs — the streaming/assembled equivalence the interop requirement asks for.
pub fn drive_batch<S: Sink>(sink: &mut S, maude: &MaudeInfo, theories: &[BatchTheory]) {
    let mut e = BatchEmitter::begin(sink, maude);
    for t in theories {
        e.closed_phases(&t.name);
        e.extra_progress(&t.extra_progress);
        e.payload(t.payload.as_deref());
        e.record_summary(t.summary.clone());
    }
    e.finish();
}

//! Output-stream model.
//!
//! Every byte the binary emits is destined for exactly one of the two standard
//! streams. Which stream a given piece of text lands on is a fixed, probed
//! property (see `workspace/BEHAVIOR.md` §10): the theory payload, the summary
//! block, help pages, the version banner, and the `error: … + full help`
//! validation envelopes go to stdout; the maude preamble, per-theory progress
//! markers, the bare cmdargs one-liners, and all runtime errors go to stderr.

/// One of the two standard output streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    /// Standard output.
    Out,
    /// Standard error.
    Err,
}

/// A pair of independently byte-exact stream buffers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Streams {
    pub out: String,
    pub err: String,
}

impl Streams {
    /// Append `text` to the buffer for `stream`.
    pub fn push(&mut self, stream: Stream, text: &str) {
        match stream {
            Stream::Out => self.out.push_str(text),
            Stream::Err => self.err.push_str(text),
        }
    }
}

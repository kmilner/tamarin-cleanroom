//! Error text renderers.
//!
//! Two tiers, all observed to exit 1, each pinned to a stream (see
//! `workspace/BEHAVIOR.md` §8, §10, §11):
//! - cmdargs-level parse errors, produced by [`crate::parse`] ([`ParseError`]):
//!   the bare one-liners (`Unknown flag`, `Ambiguous mode`, `Unhandled argument`)
//!   go to stderr; the `error: … + full help` validation envelopes go to stdout.
//! - value-validation errors and runtime errors printed with the maude preamble
//!   (integer/enum flag rejection, file-open failures, application `error` call
//!   sites) go to stderr.
//!
//! Every literal string here is compatibility output copied from observed oracle
//! output (see `workspace/captures/`), except the consumer-extension flag
//! rejections ([`bad_positive_int`], [`missing_value`]) which have no reference
//! and carry original text.

use crate::help::render_help;
use crate::modes::Mode;
use crate::stream::Stream;

/// A ready-to-print error: its text, the stream it is written to, and its exit
/// code (observed: always 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub text: String,
    pub stream: Stream,
    pub exit_code: i32,
}

/// Legacy name retained for callers that predate the stream-aware split.
pub type CliError = ParseError;

impl ParseError {
    fn new(text: String, stream: Stream) -> Self {
        ParseError { text, stream, exit_code: 1 }
    }
}

// ---- cmdargs-level parse errors (bare one-liners -> stderr) -------------------

/// `Unknown flag: <flag>` (`flag` includes its dashes).
pub fn unknown_flag(flag: &str) -> ParseError {
    ParseError::new(format!("Unknown flag: {}\n", flag), Stream::Err)
}

/// `Unhandled argument to flag, none expected: <token>` for a value given to a
/// valueless flag (`token` is the whole `--flag=value`).
pub fn unhandled_argument(token: &str) -> ParseError {
    ParseError::new(
        format!("Unhandled argument to flag, none expected: {}\n", token),
        Stream::Err,
    )
}

/// `Ambiguous mode '<token>', could be any of: <names…>`.
pub fn ambiguous_mode(token: &str, names: &[&str]) -> ParseError {
    ParseError::new(
        format!("Ambiguous mode '{}', could be any of: {}\n", token, names.join(" ")),
        Stream::Err,
    )
}

// ---- cmdargs-level validation envelopes (message + full help -> stdout) -------

/// `error: no input files given` + blank line + the mode's full help page.
pub fn no_input_files(mode: Mode) -> ParseError {
    ParseError::new(
        format!("error: no input files given\n\n{}", render_help(mode)),
        Stream::Out,
    )
}

/// `error: directory '<dir>' does not exist.` + blank line + the mode's full help
/// page. (Interactive WORKDIR check; requires the filesystem, so it is a
/// caller-invoked renderer rather than a parse-time result.)
pub fn directory_does_not_exist(dir: &str, mode: Mode) -> ParseError {
    ParseError::new(
        format!("error: directory '{}' does not exist.\n\n{}", dir, render_help(mode)),
        Stream::Out,
    )
}

// ---- value-validation errors (app `error` shape -> stderr) --------------------

/// The application-`error` block: a message wrapped with the `tamarin-prover: `
/// prefix and the observed Haskell CallStack (every value-validation and known
/// application error cites the same call site).
pub fn app_error(message: &str) -> String {
    format!(
        "tamarin-prover: {}\nCallStack (from HasCallStack):\n  error, called at src/Main/Mode/Batch.hs:162:33 in main:Main.Mode.Batch\n",
        message
    )
}

/// `<label>: invalid bound given` — a numeric flag given an unparseable value.
/// `label` is the flag's internal tag (`bound`, `OpenChainsLimit`, …).
pub fn invalid_bound(label: &str) -> ParseError {
    ParseError::new(app_error(&format!("{}: invalid bound given", label)), Stream::Err)
}

/// `unknown stop-on-trace method: <value>` — the value is the lowercased input.
pub fn unknown_stop_on_trace(lowercased_value: &str) -> ParseError {
    ParseError::new(
        app_error(&format!("unknown stop-on-trace method: {}", lowercased_value)),
        Stream::Err,
    )
}

/// `partial-evaluation: unknown option`.
pub fn partial_evaluation_unknown() -> ParseError {
    ParseError::new(app_error("partial-evaluation: unknown option"), Stream::Err)
}

/// `output mode not supported.` — an unrecognized `--output-module` value.
pub fn output_mode_not_supported() -> ParseError {
    ParseError::new(app_error("output mode not supported."), Stream::Err)
}

// ---- consumer-extension flag rejections (no reference; original text) ---------

/// A consumer-extension flag (`--processors`, `--maude-processes`) whose value is
/// not a positive integer. This flag is not present in the reference binary, so
/// the text is original rather than reproduced.
pub fn bad_positive_int(flag: &str, value: &str) -> ParseError {
    ParseError::new(
        format!("error: flag {} expects a positive integer, got '{}'\n", flag, value),
        Stream::Err,
    )
}

/// A consumer-extension flag given without the required value.
pub fn missing_value(flag: &str) -> ParseError {
    ParseError::new(format!("error: flag {} requires a value\n", flag), Stream::Err)
}

// ---- runtime file-open errors (printed after the maude preamble -> stderr) ----

/// Reason an input file could not be opened, with its exact message suffix.
#[derive(Debug, Clone, Copy)]
pub enum OpenFileError {
    DoesNotExist,
    PermissionDenied,
    IsDirectory,
}

impl OpenFileError {
    fn suffix(self) -> &'static str {
        match self {
            OpenFileError::DoesNotExist => "openFile: does not exist (No such file or directory)",
            OpenFileError::PermissionDenied => "openFile: permission denied (Permission denied)",
            OpenFileError::IsDirectory => "openFile: inappropriate type (is a directory)",
        }
    }
}

/// `tamarin-prover: <path>: <reason>` — a file-open failure line (stderr).
pub fn open_file_error(path: &str, reason: OpenFileError) -> String {
    format!("tamarin-prover: {}: {}\n", path, reason.suffix())
}

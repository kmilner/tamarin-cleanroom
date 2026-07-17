//! Error text renderers.
//!
//! Two tiers, both observed to exit 1:
//! - cmdargs-level parse errors, produced by [`crate::parse`] ([`CliError`]).
//! - runtime errors printed after the maude preamble (file-open failures and
//!   application `error` call sites); these are text producers the caller emits
//!   during processing, not parse-time results.
//!
//! Every literal string here is compatibility output copied from observed oracle
//! output (see `workspace/captures/`).

use crate::help::render_help;
use crate::modes::Mode;

/// A ready-to-print error with its exit code (observed: always 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    pub text: String,
    pub exit_code: i32,
}

impl CliError {
    fn code1(text: String) -> Self {
        CliError { text, exit_code: 1 }
    }
}

// ---- cmdargs-level parse errors ----------------------------------------------

/// `Unknown flag: <flag>` (bare one-liner; `flag` includes its dashes).
pub fn unknown_flag(flag: &str) -> CliError {
    CliError::code1(format!("Unknown flag: {}\n", flag))
}

/// `Unhandled argument to flag, none expected: <token>` for a value given to a
/// valueless flag (`token` is the whole `--flag=value`).
pub fn unhandled_argument(token: &str) -> CliError {
    CliError::code1(format!("Unhandled argument to flag, none expected: {}\n", token))
}

/// `Ambiguous mode '<token>', could be any of: <names…>`.
pub fn ambiguous_mode(token: &str, names: &[&str]) -> CliError {
    CliError::code1(format!(
        "Ambiguous mode '{}', could be any of: {}\n",
        token,
        names.join(" ")
    ))
}

/// `error: no input files given` followed by a blank line and the mode's full
/// help page.
pub fn no_input_files(mode: Mode) -> CliError {
    CliError::code1(format!("error: no input files given\n\n{}", render_help(mode)))
}

/// `error: directory '<dir>' does not exist.` followed by a blank line and the
/// mode's full help page. (Interactive WORKDIR check; requires the filesystem,
/// so it is a caller-invoked renderer rather than a parse-time result.)
pub fn directory_does_not_exist(dir: &str, mode: Mode) -> CliError {
    CliError::code1(format!(
        "error: directory '{}' does not exist.\n\n{}",
        dir,
        render_help(mode)
    ))
}

// ---- runtime errors (printed after the maude preamble) -----------------------

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

/// `tamarin-prover: <path>: <reason>` — a file-open failure line.
pub fn open_file_error(path: &str, reason: OpenFileError) -> String {
    format!("tamarin-prover: {}: {}\n", path, reason.suffix())
}

/// An application `error` call with the observed Haskell CallStack. `message` is
/// one of the known messages, e.g. `bound: invalid bound given` or
/// `output mode not supported.`.
pub fn app_error(message: &str) -> String {
    format!(
        "tamarin-prover: {}\nCallStack (from HasCallStack):\n  error, called at src/Main/Mode/Batch.hs:162:33 in main:Main.Mode.Batch\n",
        message
    )
}

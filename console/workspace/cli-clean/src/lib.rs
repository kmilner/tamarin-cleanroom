//! Clean-room reimplementation of the tamarin-prover CLI/console text surface.
//!
//! Unit D of the relicensing effort. Every observable string in this crate was
//! derived from black-box probing of the compiled binary (see
//! `workspace/BEHAVIOR.md` and `workspace/QUERIES.log`); exact output strings
//! (help pages, banners, error texts) are stored as observed-output fixtures and
//! treated as opaque compatibility content. The parsing architecture, flag-table
//! model, and framing assembly are original.
//!
//! Public surface:
//! - [`parse`] : `parse(argv) -> Result<Command, ParseError>` — the structural
//!   command (mode, positionals, raw flag occurrences), or ready-to-print error.
//! - [`parse_args`] : `parse_args(argv) -> Result<Parsed, ParseError>` — a typed,
//!   value-validated command line ([`Args`]) with defaults applied.
//! - [`render_help`] / [`render_version`] : the help pages and version banner.
//! - [`framing`] : the stream-aware batch frame around opaque theory payloads.
//! - [`errors`] : renderers for the runtime error lines emitted after parsing.
//! - [`stream`] : the two-stream output model ([`Stream`], [`Streams`]).

pub mod args;
pub mod errors;
pub mod framing;
pub mod help;
pub mod modes;
pub mod parse;
pub mod stream;
pub mod version;

pub use args::{parse_args, Args, OutputModule, Parsed, PartialEval, StopOnTrace};
pub use errors::{CliError, ParseError};
pub use help::render_help;
pub use modes::Mode;
pub use parse::{parse, Command, Options, RunSpec};
pub use stream::{Stream, Streams};
pub use version::{frame_version, render_version, VersionInfo};

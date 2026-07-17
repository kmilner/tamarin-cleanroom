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
//! - [`parse`] : `parse(argv) -> Result<Command, CliError>` — the structured
//!   command, or ready-to-print error text.
//! - [`render_help`] / [`render_version`] : the help pages and version banner.
//! - [`framing`] : the batch-mode output frame around an opaque theory payload.
//! - [`errors`] : renderers for the runtime error lines emitted after parsing.

pub mod modes;
pub mod parse;
pub mod help;
pub mod version;
pub mod framing;
pub mod errors;

pub use errors::CliError;
pub use help::render_help;
pub use modes::Mode;
pub use parse::{parse, Command, Options, RunSpec};
pub use version::{render_version, VersionInfo};

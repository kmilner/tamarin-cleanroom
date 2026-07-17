//! `parse(argv) -> Result<Command, CliError>`.
//!
//! Turns an argument vector into a structured [`Command`] or ready-to-print
//! error text. Parsing is pure (no filesystem access): the two cmdargs-level
//! errors that require the filesystem — the interactive WORKDIR existence check —
//! are surfaced as a [`RunSpec`] the caller can validate with
//! [`crate::errors::directory_does_not_exist`]. The "no input files given" check
//! needs no filesystem and is emitted here.

use crate::errors::{self, CliError};
use crate::modes::{flags_for, FlagSpec, Mode};

/// A structured, actionable command produced by [`parse`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Print the given mode's help page (exit 0).
    Help(Mode),
    /// Print the version banner (exit 0). Only reachable in the default mode.
    Version,
    /// A mode invocation with parsed positionals and options.
    Run(RunSpec),
}

/// A parsed mode invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSpec {
    pub mode: Mode,
    /// FILES (batch/test) or the WORKDIR token(s) (interactive).
    pub positional: Vec<String>,
    pub options: Options,
}

/// Recognized flags recorded in encounter order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    /// `(canonical_long_name, value)` per occurrence; `value` is `None` for the
    /// bare form of a value flag and for boolean flags.
    pub flags: Vec<(String, Option<String>)>,
}

impl Options {
    /// True if the boolean/valued flag with this canonical long name was seen.
    pub fn is_set(&self, long: &str) -> bool {
        self.flags.iter().any(|(n, _)| n == long)
    }
    /// All values supplied for a (possibly repeated) flag.
    pub fn values(&self, long: &str) -> Vec<&str> {
        self.flags
            .iter()
            .filter(|(n, _)| n == long)
            .filter_map(|(_, v)| v.as_deref())
            .collect()
    }
    /// The last occurrence of a flag (last-wins semantics): `None` if the flag is
    /// absent, `Some(None)` if its last occurrence was bare, `Some(Some(v))` if
    /// its last occurrence carried the value `v`.
    pub fn last(&self, long: &str) -> Option<Option<&str>> {
        self.flags
            .iter()
            .rev()
            .find(|(n, _)| n == long)
            .map(|(_, v)| v.as_deref())
    }
    /// Every occurrence of a repeatable flag, in order; each element is `None` for
    /// a bare occurrence or `Some(v)` for a valued one.
    pub fn occurrences(&self, long: &str) -> Vec<Option<String>> {
        self.flags
            .iter()
            .filter(|(n, _)| n == long)
            .map(|(_, v)| v.clone())
            .collect()
    }
}

/// Parse an argument vector (excluding argv[0]/program name).
pub fn parse(argv: &[String]) -> Result<Command, CliError> {
    // 1. Determine mode and the slice of tokens to parse within it.
    let (mode, rest): (Mode, &[String]) = match argv.first() {
        // Truly empty argv is not observable through the oracle; treated as the
        // default-mode no-input case (documented inference).
        None => return Err(errors::no_input_files(Mode::Batch)),
        Some(first) if first.starts_with('-') => (Mode::Batch, argv),
        Some(first) => {
            let matches = Mode::prefix_matches(first);
            match matches.len() {
                1 => (matches[0].1, &argv[1..]),
                0 => (Mode::Batch, argv), // not a mode prefix -> a FILE
                _ => {
                    let names: Vec<&str> = matches.iter().map(|(n, _)| *n).collect();
                    return Err(errors::ambiguous_mode(first, &names));
                }
            }
        }
    };

    // 2. Left-to-right token scan within the chosen mode.
    let table = flags_for(mode);
    let mut options = Options::default();
    let mut positional: Vec<String> = Vec::new();

    let mut i = 0;
    while i < rest.len() {
        let tok = &rest[i];
        if let Some(body) = tok.strip_prefix("--") {
            // Long flag: `--name` or `--name=value`.
            let (name, mut value) = match body.split_once('=') {
                Some((n, v)) => (n, Some(v.to_string())),
                None => (body, None),
            };
            let spec = lookup_long(&table, name)
                .ok_or_else(|| errors::unknown_flag(&format!("--{name}")))?;
            if !spec.takes_value && value.is_some() {
                return Err(errors::unhandled_argument(tok));
            }
            // A consumer-extension flag with no attached value takes the following
            // token as its value (the `--flag <value>` surface).
            if spec.consumes_next && value.is_none() {
                if let Some(next) = rest.get(i + 1) {
                    value = Some(next.clone());
                    i += 1;
                } else {
                    return Err(errors::missing_value(&format!("--{name}")));
                }
            }
            if let Some(c) = short_circuit(spec.long, mode) {
                return Ok(c);
            }
            options.flags.push((spec.long.to_string(), value));
        } else if tok.len() >= 2 && tok.starts_with('-') && tok != "--" {
            // Short flag: `-x` or `-x<attached value>`.
            let mut chars = tok[1..].chars();
            let c = chars.next().unwrap();
            let attached: String = chars.collect();
            let spec = lookup_short(&table, c)
                .ok_or_else(|| errors::unknown_flag(tok))?;
            if spec.takes_value {
                let value = if attached.is_empty() { None } else { Some(attached) };
                if let Some(cmd) = short_circuit(spec.long, mode) {
                    return Ok(cmd);
                }
                options.flags.push((spec.long.to_string(), value));
            } else {
                // Boolean short flag: only the bare form is recognized here.
                if !attached.is_empty() {
                    return Err(errors::unknown_flag(tok));
                }
                if let Some(cmd) = short_circuit(spec.long, mode) {
                    return Ok(cmd);
                }
                options.flags.push((spec.long.to_string(), None));
            }
        } else {
            // Positional token (a FILE, or the WORKDIR for interactive).
            positional.push(tok.clone());
        }
        i += 1;
    }

    // 3. Positional-arity validation that cmdargs performs at parse time.
    // Batch and test require at least one input FILE.
    if matches!(mode, Mode::Batch | Mode::Test) && positional.is_empty() {
        return Err(errors::no_input_files(mode));
    }

    Ok(Command::Run(RunSpec { mode, positional, options }))
}

/// Map an About flag to its terminal command, if it is one.
fn short_circuit(long: &str, mode: Mode) -> Option<Command> {
    match long {
        "help" => Some(Command::Help(mode)),
        "version" => Some(Command::Version),
        _ => None,
    }
}

fn lookup_long<'a>(table: &'a [FlagSpec], name: &str) -> Option<&'a FlagSpec> {
    table
        .iter()
        .find(|f| f.long == name || f.aliases.contains(&name))
}

fn lookup_short(table: &[FlagSpec], c: char) -> Option<&FlagSpec> {
    table.iter().find(|f| f.short == Some(c))
}

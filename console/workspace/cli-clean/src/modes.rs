//! Modes and their flag tables.
//!
//! The binary is a multi-mode command. The first token selects a mode by unique
//! prefix match against the mode names; otherwise the default (batch) mode is
//! used and the token is a positional. Each mode recognizes a different set of
//! flags, so the flag table is mode-scoped (a flag valid in one mode is
//! "Unknown" in another — e.g. `--version` outside the default mode).

/// The four invocation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Default mode when no mode token is given: analyze/prove theory FILES.
    Batch,
    /// `interactive` — web server; positional WORKDIR.
    Interactive,
    /// `variants` — compute intruder-rule variants; no positional.
    Variants,
    /// `test` — self-test; positional FILES.
    Test,
}

impl Mode {
    /// The three named modes, in the order the help/ambiguity messages list them.
    pub const NAMED: [(&'static str, Mode); 3] = [
        ("interactive", Mode::Interactive),
        ("variants", Mode::Variants),
        ("test", Mode::Test),
    ];

    /// Modes whose named token is a prefix of `token` (empty token matches all).
    pub fn prefix_matches(token: &str) -> Vec<(&'static str, Mode)> {
        Mode::NAMED
            .iter()
            .filter(|(name, _)| name.starts_with(token))
            .copied()
            .collect()
    }
}

/// One recognized flag: its canonical long name, optional short char, whether it
/// accepts a value, and any additional long aliases.
#[derive(Debug, Clone, Copy)]
pub struct FlagSpec {
    pub long: &'static str,
    pub short: Option<char>,
    pub takes_value: bool,
    pub aliases: &'static [&'static str],
}

const fn boolf(long: &'static str, short: Option<char>) -> FlagSpec {
    FlagSpec { long, short, takes_value: false, aliases: &[] }
}
const fn valf(long: &'static str, short: Option<char>) -> FlagSpec {
    FlagSpec { long, short, takes_value: true, aliases: &[] }
}

// The proving/output flag subset shared by the default and interactive modes.
const PROVE_COMMON: &[FlagSpec] = &[
    valf("prove", None),
    valf("lemma", None),
    valf("stop-on-trace", None),
    valf("bound", Some('b')),
    valf("heuristic", None),
    valf("partial-evaluation", None),
    valf("defines", Some('D')),
    boolf("diff", None),
    boolf("quit-on-warning", None),
    boolf("auto-sources", None),
    valf("oraclename", None),
    boolf("oracle-only", None),
    boolf("quiet", None),
    boolf("verbose", Some('v')),
    valf("open-chains", Some('c')),
    valf("saturation", Some('s')),
    valf("derivcheck-timeout", Some('d')),
    boolf("no-reuse", None),
    boolf("no-restrictions", None),
    valf("replication-bound", None),
];

const WITH_TOOLS: &[FlagSpec] = &[
    valf("with-dot", None),
    valf("with-json", None),
    valf("with-maude", None),
];

/// Flags recognized in the default/batch mode (excluding About flags).
pub fn batch_flags() -> Vec<FlagSpec> {
    let mut v: Vec<FlagSpec> = PROVE_COMMON.to_vec();
    v.extend_from_slice(&[
        boolf("no-compress", None),
        boolf("parse-only", None),
        boolf("precompute-only", None),
        valf("output", Some('o')),
        valf("Output", Some('O')),
        valf("output-module", Some('m')),
        FlagSpec { long: "output-json", short: None, takes_value: true, aliases: &["oj"] },
        FlagSpec { long: "output-dot", short: None, takes_value: true, aliases: &["od"] },
    ]);
    v.extend_from_slice(WITH_TOOLS);
    v
}

/// Flags recognized in the interactive mode (excluding About flags).
pub fn interactive_flags() -> Vec<FlagSpec> {
    let mut v = vec![
        valf("port", Some('p')),
        valf("interface", Some('i')),
        valf("image-format", None),
        boolf("debug", None),
        boolf("no-logging", None),
    ];
    v.extend_from_slice(PROVE_COMMON);
    v.extend_from_slice(WITH_TOOLS);
    v
}

/// Flags recognized in the variants mode (excluding About flags).
pub fn variants_flags() -> Vec<FlagSpec> {
    vec![valf("Output", Some('O'))]
}

/// Flags recognized in the test mode (excluding About flags).
pub fn test_flags() -> Vec<FlagSpec> {
    WITH_TOOLS.to_vec()
}

/// The mode-scoped flag table (mode flags first, then About flags). `--version`
/// / `-V` is an About flag only in the default mode.
pub fn flags_for(mode: Mode) -> Vec<FlagSpec> {
    let mut v = match mode {
        Mode::Batch => batch_flags(),
        Mode::Interactive => interactive_flags(),
        Mode::Variants => variants_flags(),
        Mode::Test => test_flags(),
    };
    // About: help everywhere; version only in the default mode.
    v.push(FlagSpec { long: "help", short: Some('?'), takes_value: false, aliases: &[] });
    if mode == Mode::Batch {
        v.push(boolf("version", Some('V')));
    }
    v
}

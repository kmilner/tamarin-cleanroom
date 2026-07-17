//! Typed argument parsing with value validation.
//!
//! [`parse_args`] layers a typed surface over the structural [`crate::parse`]:
//! `parse_args(argv) -> Result<Parsed, ParseError>`. On success it yields either a
//! help/version request or a fully typed [`Args`] with integers parsed, enums
//! resolved, and documented defaults applied. On a value-validation failure it
//! yields a [`ParseError`] whose text and stream reproduce the reference binary's
//! output byte-for-byte (see `workspace/BEHAVIOR.md` §11).
//!
//! Only eight flags are value-validated (the rest accept any value). Validation
//! fires only for a *present* value (`=value`, `--oj=…`, or short-attached `-b5`);
//! a bare flag never fails. Repeated flags follow last-wins. When several flags
//! carry an invalid value, a fixed order — independent of command-line order —
//! decides which error surfaces:
//!
//! `stop-on-trace > bound > partial-evaluation > open-chains > saturation >
//! output-module > derivcheck-timeout > replication-bound`.

use crate::errors::{self, ParseError};
use crate::modes::Mode;
use crate::parse::{parse, Command, Options};

/// The `--stop-on-trace` method (matched case-insensitively).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopOnTrace {
    Dfs,
    Bfs,
    SeqDfs,
    Sorry,
    None,
}

/// The `--partial-evaluation` mode (matched case-insensitively).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialEval {
    Summary,
    Verbose,
}

/// The `-m --output-module` target (matched case-sensitively).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputModule {
    SpthyTyped,
    Spthy,
    Msr,
    ProverifEquiv,
    Proverif,
    Deepsec,
}

/// Documented default for `--open-chains`.
pub const OPEN_CHAINS_DEFAULT: i64 = 10;
/// Documented default for `--saturation`.
pub const SATURATION_DEFAULT: i64 = 5;
/// Documented default for `--derivcheck-timeout`.
pub const DERIVCHECK_TIMEOUT_DEFAULT: i64 = 5;

/// A fully typed, validated command-line, with defaults applied.
///
/// Fields cover every flag the mode tables model. Flags irrelevant to the active
/// [`Mode`] simply hold their defaults. Bare-vs-valued is preserved only where it
/// is semantically load-bearing (the repeatable selectors); other single-valued
/// string flags collapse an absent or bare occurrence to `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub mode: Mode,
    /// FILES (batch/test) or the WORKDIR token(s) (interactive).
    pub positional: Vec<String>,

    // -- proving / lemma selection (repeatable; None element = bare occurrence) --
    pub prove: Vec<Option<String>>,
    pub lemma: Vec<Option<String>>,
    pub defines: Vec<Option<String>>,

    // -- value-validated flags --
    pub stop_on_trace: StopOnTrace,
    pub bound: Option<i64>,
    pub partial_evaluation: Option<PartialEval>,
    pub open_chains: i64,
    pub saturation: i64,
    pub output_module: Option<OutputModule>,
    pub derivcheck_timeout: i64,
    pub replication_bound: Option<i64>,

    // -- unvalidated value flags (raw last-occurrence value) --
    pub heuristic: Option<String>,
    pub oraclename: Option<String>,
    pub output: Option<String>,
    pub output_dir: Option<String>,
    pub output_json: Option<String>,
    pub output_dot: Option<String>,
    pub with_dot: Option<String>,
    pub with_json: Option<String>,
    pub with_maude: Option<String>,
    // interactive-only value flags (unvalidated by the reference)
    pub port: Option<String>,
    pub interface: Option<String>,
    pub image_format: Option<String>,

    // -- boolean flags --
    pub diff: bool,
    pub quit_on_warning: bool,
    pub auto_sources: bool,
    pub oracle_only: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub no_reuse: bool,
    pub no_restrictions: bool,
    pub no_compress: bool,
    pub parse_only: bool,
    pub precompute_only: bool,
    pub debug: bool,
    pub no_logging: bool,

    // -- consumer-extension flags (not in the reference, not in help) --
    pub processors: Option<u64>,
    pub maude_processes: Option<u64>,
    pub data_dir: Option<String>,
}

/// The outcome of a typed parse: a help/version request, or a run with typed args.
///
/// `Run` is the common case and the value is consumed immediately, so the size
/// gap between variants carries no practical cost.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsed {
    Help(Mode),
    Version,
    Run(Args),
}

/// Typed parse of an argument vector (excluding argv[0]).
pub fn parse_args(argv: &[String]) -> Result<Parsed, ParseError> {
    match parse(argv)? {
        Command::Help(mode) => Ok(Parsed::Help(mode)),
        Command::Version => Ok(Parsed::Version),
        Command::Run(spec) => {
            let args = build_args(spec.mode, spec.positional, &spec.options)?;
            Ok(Parsed::Run(args))
        }
    }
}

/// Resolve typed fields, validating values in the fixed precedence order so the
/// first invalid flag in that order produces the surfaced error.
fn build_args(mode: Mode, positional: Vec<String>, o: &Options) -> Result<Args, ParseError> {
    // Precedence order: the earliest invalid flag here is the one that errors.
    let stop_on_trace = stop_on_trace_field(o)?;
    let bound = int_opt(o, "bound", "bound")?;
    let partial_evaluation = partial_eval_field(o)?;
    let open_chains = int_default(o, "open-chains", "OpenChainsLimit", OPEN_CHAINS_DEFAULT)?;
    let saturation = int_default(o, "saturation", "SaturationLimit", SATURATION_DEFAULT)?;
    let output_module = output_module_field(o)?;
    let derivcheck_timeout =
        int_default(o, "derivcheck-timeout", "derivcheck-timeout", DERIVCHECK_TIMEOUT_DEFAULT)?;
    let replication_bound = int_opt(o, "replication-bound", "replication-bound")?;

    // Consumer extensions (validated after all reference flags, so a reference
    // error always wins).
    let processors = positive_int_opt(o, "processors", "--processors")?;
    let maude_processes = positive_int_opt(o, "maude-processes", "--maude-processes")?;

    Ok(Args {
        mode,
        positional,
        prove: o.occurrences("prove"),
        lemma: o.occurrences("lemma"),
        defines: o.occurrences("defines"),
        stop_on_trace,
        bound,
        partial_evaluation,
        open_chains,
        saturation,
        output_module,
        derivcheck_timeout,
        replication_bound,
        heuristic: raw_opt(o, "heuristic"),
        oraclename: raw_opt(o, "oraclename"),
        output: raw_opt(o, "output"),
        output_dir: raw_opt(o, "Output"),
        output_json: raw_opt(o, "output-json"),
        output_dot: raw_opt(o, "output-dot"),
        with_dot: raw_opt(o, "with-dot"),
        with_json: raw_opt(o, "with-json"),
        with_maude: raw_opt(o, "with-maude"),
        port: raw_opt(o, "port"),
        interface: raw_opt(o, "interface"),
        image_format: raw_opt(o, "image-format"),
        diff: o.is_set("diff"),
        quit_on_warning: o.is_set("quit-on-warning"),
        auto_sources: o.is_set("auto-sources"),
        oracle_only: o.is_set("oracle-only"),
        quiet: o.is_set("quiet"),
        verbose: o.is_set("verbose"),
        no_reuse: o.is_set("no-reuse"),
        no_restrictions: o.is_set("no-restrictions"),
        no_compress: o.is_set("no-compress"),
        parse_only: o.is_set("parse-only"),
        precompute_only: o.is_set("precompute-only"),
        debug: o.is_set("debug"),
        no_logging: o.is_set("no-logging"),
        processors,
        maude_processes,
        data_dir: raw_opt(o, "data-dir"),
    })
}

/// Last-occurrence raw value (absent or bare → `None`).
fn raw_opt(o: &Options, long: &str) -> Option<String> {
    match o.last(long) {
        Some(Some(v)) => Some(v.to_string()),
        _ => None,
    }
}

/// Integer flag with no documented default: `None` when absent/bare, else the
/// parsed value or the flag's `invalid bound` error.
fn int_opt(o: &Options, long: &str, label: &str) -> Result<Option<i64>, ParseError> {
    match o.last(long) {
        Some(Some(v)) => match read_haskell_int(v) {
            Some(n) => Ok(Some(n)),
            None => Err(errors::invalid_bound(label)),
        },
        _ => Ok(None),
    }
}

/// Integer flag with a documented default applied for absent/bare occurrences.
fn int_default(o: &Options, long: &str, label: &str, default: i64) -> Result<i64, ParseError> {
    match o.last(long) {
        Some(Some(v)) => read_haskell_int(v).ok_or_else(|| errors::invalid_bound(label)),
        _ => Ok(default),
    }
}

/// `--stop-on-trace`: case-insensitive; default DFS; invalid value echoes the
/// lowercased input.
fn stop_on_trace_field(o: &Options) -> Result<StopOnTrace, ParseError> {
    match o.last("stop-on-trace") {
        Some(Some(v)) => {
            let lower = v.to_ascii_lowercase();
            match lower.as_str() {
                "dfs" => Ok(StopOnTrace::Dfs),
                "bfs" => Ok(StopOnTrace::Bfs),
                "seqdfs" => Ok(StopOnTrace::SeqDfs),
                "sorry" => Ok(StopOnTrace::Sorry),
                "none" => Ok(StopOnTrace::None),
                _ => Err(errors::unknown_stop_on_trace(&lower)),
            }
        }
        _ => Ok(StopOnTrace::Dfs),
    }
}

/// `--partial-evaluation`: case-insensitive; no default; no value echoed on error.
fn partial_eval_field(o: &Options) -> Result<Option<PartialEval>, ParseError> {
    match o.last("partial-evaluation") {
        Some(Some(v)) => match v.to_ascii_lowercase().as_str() {
            "summary" => Ok(Some(PartialEval::Summary)),
            "verbose" => Ok(Some(PartialEval::Verbose)),
            _ => Err(errors::partial_evaluation_unknown()),
        },
        _ => Ok(None),
    }
}

/// `-m --output-module`: case-SENSITIVE; no value echoed on error.
fn output_module_field(o: &Options) -> Result<Option<OutputModule>, ParseError> {
    match o.last("output-module") {
        Some(Some(v)) => match v {
            "spthytyped" => Ok(Some(OutputModule::SpthyTyped)),
            "spthy" => Ok(Some(OutputModule::Spthy)),
            "msr" => Ok(Some(OutputModule::Msr)),
            "proverifequiv" => Ok(Some(OutputModule::ProverifEquiv)),
            "proverif" => Ok(Some(OutputModule::Proverif)),
            "deepsec" => Ok(Some(OutputModule::Deepsec)),
            _ => Err(errors::output_mode_not_supported()),
        },
        _ => Ok(None),
    }
}

/// A consumer-extension positive-integer flag: `None` when absent/bare, else a
/// value ≥ 1 or an original (non-reference) rejection.
fn positive_int_opt(o: &Options, long: &str, flag: &str) -> Result<Option<u64>, ParseError> {
    match o.last(long) {
        Some(Some(v)) => match v.trim().parse::<u64>() {
            Ok(n) if n >= 1 => Ok(Some(n)),
            _ => Err(errors::bad_positive_int(flag, v)),
        },
        _ => Ok(None),
    }
}

/// Parse an integer the way Haskell's `readMaybe :: String -> Maybe Int` does, to
/// match the reference's accept/reject boundary (see `workspace/BEHAVIOR.md` §11):
/// trims surrounding whitespace, allows a leading `-` with optional whitespace
/// before the digits, accepts decimal / `0x`hex / `0o`octal, and rejects `+`, a
/// decimal point, underscores, trailing non-whitespace, and the empty string.
/// Values beyond the 64-bit range wrap rather than fail, mirroring `fromInteger`.
pub fn read_haskell_int(s: &str) -> Option<i64> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let (negative, mag) = match t.strip_prefix('-') {
        Some(rest) => (true, rest.trim_start()),
        None => (false, t),
    };
    if mag.is_empty() {
        return None;
    }
    let (digits, radix) = if let Some(h) = mag.strip_prefix("0x").or_else(|| mag.strip_prefix("0X"))
    {
        (h, 16u32)
    } else if let Some(oc) = mag.strip_prefix("0o").or_else(|| mag.strip_prefix("0O")) {
        (oc, 8u32)
    } else {
        (mag, 10u32)
    };
    if digits.is_empty() {
        return None;
    }
    // Accumulate with wrapping so out-of-range inputs mirror Haskell's wrap rather
    // than being rejected.
    let mut acc: i64 = 0;
    for ch in digits.chars() {
        let d = ch.to_digit(radix)?;
        acc = acc.wrapping_mul(radix as i64).wrapping_add(d as i64);
    }
    Some(if negative { acc.wrapping_neg() } else { acc })
}

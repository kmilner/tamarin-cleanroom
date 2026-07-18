//! Byte-parity tests against captured oracle output.
//!
//! Fixtures in `tests/fixtures/` are raw captures of the black-box binary. The
//! `split_*` fixtures were captured with stdout and stderr redirected separately
//! (`1>out 2>err`); the framing tests reassemble BOTH streams and compare them
//! byte-for-byte. The `vv_*` fixtures are the value-validation error texts.

use cli_clean::args::{parse_args, read_haskell_int, Args, OutputModule, Parsed, PartialEval, StopOnTrace};
use cli_clean::emit::{drive_batch, BatchEmitter, StreamCollector};
use cli_clean::errors::{app_error, open_file_error, OpenFileError, ParseError};
use cli_clean::framing::{
    frame_batch, frame_parse_only, frame_variants, render_summary, BatchTheory, LemmaOutcome,
    LemmaResult, LemmaSide, LoadedTheory, Summary, TraceKind, WarningSummary,
};
use cli_clean::stream::{Stream, Streams};
use cli_clean::version::{frame_version, MaudeInfo, VersionInfo};
use cli_clean::{parse, render_help, render_version, Command, Mode};

fn s(a: &str) -> String {
    a.to_string()
}
fn argv(items: &[&str]) -> Vec<String> {
    items.iter().map(|x| x.to_string()).collect()
}
fn run_err(items: &[&str]) -> ParseError {
    parse_args(&argv(items)).unwrap_err()
}
fn run_ok(items: &[&str]) -> Args {
    match parse_args(&argv(items)).unwrap() {
        Parsed::Run(a) => a,
        other => panic!("expected Run, got {other:?}"),
    }
}

// ---- help ---------------------------------------------------------------------

#[test]
fn help_pages_are_byte_exact() {
    assert_eq!(render_help(Mode::Batch), include_str!("fixtures/help_global.txt"));
    assert_eq!(render_help(Mode::Interactive), include_str!("fixtures/help_interactive.txt"));
    assert_eq!(render_help(Mode::Variants), include_str!("fixtures/help_variants.txt"));
    assert_eq!(render_help(Mode::Test), include_str!("fixtures/help_test.txt"));
}

// ---- version ------------------------------------------------------------------

fn observed_version() -> VersionInfo {
    VersionInfo {
        maude: MaudeInfo::default(),
        tamarin_version: s("1.13.0"),
        copyright_years: s("2010-2023"),
        git_description: s(
            "0234f6a1abee25677c0accef62de0ac2883b0347 (with uncommited changes), branch: HEAD",
        ),
        compiled_at: s("2026-07-16 18:33:16.036427196 UTC"),
    }
}

#[test]
fn version_streams_are_byte_exact() {
    // The banner belongs on stdout; the maude preamble belongs on stderr. Only the
    // merged oracle interleaved them.
    let streams = frame_version(&observed_version());
    assert_eq!(streams.out, include_str!("fixtures/split_version.out.txt"));
    assert_eq!(streams.err, include_str!("fixtures/split_version.err.txt"));
    assert_eq!(render_version(&observed_version()), include_str!("fixtures/split_version.out.txt"));
}

// ---- structural parse errors (unchanged surface) ------------------------------

#[test]
fn unknown_long_flag() {
    let e = parse(&argv(&["--foobar"])).unwrap_err();
    assert_eq!(e.text, include_str!("fixtures/err_unknown_flag.txt"));
    assert_eq!(e.exit_code, 1);
}

#[test]
fn unknown_short_flag() {
    let e = parse(&argv(&["-Z"])).unwrap_err();
    assert_eq!(e.text, include_str!("fixtures/err_unknown_short.txt"));
}

#[test]
fn short_h_is_not_help() {
    let e = parse(&argv(&["-h"])).unwrap_err();
    assert_eq!(e.text, "Unknown flag: -h\n");
}

#[test]
fn value_on_valueless_flag() {
    let e = parse(&argv(&["--help=foo"])).unwrap_err();
    assert_eq!(e.text, include_str!("fixtures/help_eq.txt"));
}

#[test]
fn ambiguous_empty_mode() {
    let e = parse(&argv(&[""])).unwrap_err();
    assert_eq!(e.text, include_str!("fixtures/noargs.txt"));
}

#[test]
fn no_input_files_envelope_includes_global_help() {
    let e = parse(&argv(&["--stop-on-trace=XYZ"])).unwrap_err();
    assert_eq!(e.text, include_str!("fixtures/err_stopontrace_bad.txt"));
    let e2 = parse(&argv(&["--diff"])).unwrap_err();
    assert_eq!(e2.text, include_str!("fixtures/err_stopontrace_bad.txt"));
}

#[test]
fn version_flag_unknown_outside_default_mode() {
    let e = parse(&argv(&["variants", "--version"])).unwrap_err();
    assert_eq!(e.text, "Unknown flag: --version\n");
}

// ---- error stream assignment (GAP 2) ------------------------------------------

#[test]
fn error_streams_are_assigned() {
    // Bare one-liner cmdargs errors -> stderr.
    assert_eq!(run_err(&["--foobar"]).stream, Stream::Err);
    assert_eq!(run_err(&[""]).stream, Stream::Err);
    assert_eq!(run_err(&["--help=foo"]).stream, Stream::Err);
    // "error: … + full help" validation envelopes -> stdout.
    assert_eq!(run_err(&["--diff"]).stream, Stream::Out);
    // Value-validation errors -> stderr.
    assert_eq!(run_err(&["file.spthy", "--bound=x"]).stream, Stream::Err);
}

// ---- typed parse: help/version short-circuits ---------------------------------

#[test]
fn parse_args_help_and_version() {
    assert_eq!(parse_args(&argv(&["--help"])).unwrap(), Parsed::Help(Mode::Batch));
    assert_eq!(parse_args(&argv(&["int", "--help"])).unwrap(), Parsed::Help(Mode::Interactive));
    assert_eq!(parse_args(&argv(&["--version"])).unwrap(), Parsed::Version);
    assert_eq!(parse_args(&argv(&["-V"])).unwrap(), Parsed::Version);
}

// ---- GAP 1: value-validation error taxonomy -----------------------------------

#[test]
fn integer_flag_errors_match_reference() {
    let f = "file.spthy";
    assert_eq!(run_err(&[f, "--bound=x"]).text, include_str!("fixtures/vv_bound_x.err.txt"));
    assert_eq!(
        run_err(&[f, "--open-chains=x"]).text,
        include_str!("fixtures/vv_openchains_x.err.txt")
    );
    assert_eq!(
        run_err(&[f, "--saturation=x"]).text,
        include_str!("fixtures/vv_saturation_x.err.txt")
    );
    assert_eq!(
        run_err(&[f, "--derivcheck-timeout=x"]).text,
        include_str!("fixtures/vv_derivcheck_x.err.txt")
    );
    assert_eq!(
        run_err(&[f, "--replication-bound=x"]).text,
        include_str!("fixtures/vv_replication_x.err.txt")
    );
    // Same error for a short-attached bad value.
    assert_eq!(run_err(&[f, "-bx"]).text, include_str!("fixtures/vv_bound_x.err.txt"));
    // Empty '=' value is a present value and fails too.
    assert_eq!(run_err(&[f, "--bound="]).text, include_str!("fixtures/vv_bound_x.err.txt"));
}

#[test]
fn enum_flag_errors_match_reference() {
    let f = "file.spthy";
    assert_eq!(
        run_err(&[f, "--stop-on-trace=XYZ"]).text,
        include_str!("fixtures/vv_stopontrace_xyz.err.txt")
    );
    // Value is lowercased AND echoed; empty value echoes empty.
    assert_eq!(
        run_err(&[f, "--stop-on-trace="]).text,
        include_str!("fixtures/vv_stopontrace_empty.err.txt")
    );
    assert_eq!(
        run_err(&[f, "--partial-evaluation=XYZ"]).text,
        include_str!("fixtures/vv_partialeval_bad.err.txt")
    );
    assert_eq!(
        run_err(&[f, "--output-module=bogus"]).text,
        include_str!("fixtures/vv_outputmodule_bad.err.txt")
    );
}

#[test]
fn heuristic_is_not_validated() {
    // Any heuristic string is accepted (reference exits 0 for --heuristic=Z).
    let a = run_ok(&["file.spthy", "--heuristic=Z"]);
    assert_eq!(a.heuristic, Some(s("Z")));
}

#[test]
fn valid_enum_values_and_case_rules() {
    // stop-on-trace & partial-evaluation are case-insensitive.
    assert_eq!(run_ok(&["f", "--stop-on-trace=DFS"]).stop_on_trace, StopOnTrace::Dfs);
    assert_eq!(run_ok(&["f", "--stop-on-trace=seqdfs"]).stop_on_trace, StopOnTrace::SeqDfs);
    assert_eq!(run_ok(&["f", "--partial-evaluation=Summary"]).partial_evaluation, Some(PartialEval::Summary));
    // output-module is case-SENSITIVE: lowercase accepted, uppercase rejected.
    assert_eq!(run_ok(&["f", "--output-module=msr"]).output_module, Some(OutputModule::Msr));
    assert!(parse_args(&argv(&["f", "--output-module=MSR"])).is_err());
}

#[test]
fn haskell_int_accept_reject_matches_reference() {
    // Accepted by the reference (exit 0):
    for good in ["-5", "0", "5", " 5", "5 ", "  5", "- 5", "0x10", "010", "\t5", "5\n"] {
        assert!(read_haskell_int(good).is_some(), "should accept {good:?}");
    }
    // A value beyond i64 wraps rather than being rejected.
    assert!(read_haskell_int("99999999999999999999999").is_some());
    // Rejected by the reference (invalid bound given):
    for bad in ["", "x", "3.5", "5abc", "+5", "5_0", "-", " "] {
        assert!(read_haskell_int(bad).is_none(), "should reject {bad:?}");
    }
    // Values:
    assert_eq!(read_haskell_int("5"), Some(5));
    assert_eq!(read_haskell_int("-5"), Some(-5));
    assert_eq!(read_haskell_int(" 42 "), Some(42));
    assert_eq!(read_haskell_int("0x10"), Some(16));
    assert_eq!(read_haskell_int("- 7"), Some(-7));
}

#[test]
fn validation_precedence_is_fixed_and_arg_order_independent() {
    let f = "file.spthy";
    // stop-on-trace outranks bound regardless of order.
    let a = run_err(&[f, "--bound=x", "--stop-on-trace=y"]).text;
    let b = run_err(&[f, "--stop-on-trace=y", "--bound=x"]).text;
    assert_eq!(a, b);
    assert_eq!(a, app_error("unknown stop-on-trace method: y"));
    // bound outranks open-chains outranks output-module.
    assert_eq!(
        run_err(&[f, "--output-module=z", "--bound=x"]).text,
        app_error("bound: invalid bound given")
    );
    assert_eq!(
        run_err(&[f, "--output-module=z", "--open-chains=x"]).text,
        app_error("OpenChainsLimit: invalid bound given")
    );
    // output-module outranks derivcheck-timeout and replication-bound.
    assert_eq!(
        run_err(&[f, "--derivcheck-timeout=x", "--output-module=z"]).text,
        app_error("output mode not supported.")
    );
}

#[test]
fn repeated_flag_last_occurrence_wins() {
    let f = "file.spthy";
    // First bad, last good -> OK; last value wins.
    assert_eq!(run_ok(&[f, "--bound=x", "--bound=2"]).bound, Some(2));
    // Last bad -> error.
    assert!(parse_args(&argv(&[f, "--bound=2", "--bound=x"])).is_err());
    assert_eq!(run_ok(&[f, "--stop-on-trace=bad", "--stop-on-trace=dfs"]).stop_on_trace, StopOnTrace::Dfs);
}

#[test]
fn bare_flag_never_validates() {
    let f = "file.spthy";
    // Bare (no '=') is always accepted; typed field falls back to default/None.
    assert_eq!(run_ok(&[f, "--bound"]).bound, None);
    assert_eq!(run_ok(&[f, "--stop-on-trace"]).stop_on_trace, StopOnTrace::Dfs);
    assert_eq!(run_ok(&[f, "--open-chains"]).open_chains, 10);
    // '=' with empty value is a present value and IS validated.
    assert!(parse_args(&argv(&[f, "--bound="])).is_err());
}

#[test]
fn defaults_applied_when_absent() {
    let a = run_ok(&["file.spthy"]);
    assert_eq!(a.stop_on_trace, StopOnTrace::Dfs);
    assert_eq!(a.bound, None);
    assert_eq!(a.open_chains, 10);
    assert_eq!(a.saturation, 5);
    assert_eq!(a.derivcheck_timeout, 5);
    assert_eq!(a.partial_evaluation, None);
    assert_eq!(a.output_module, None);
    assert_eq!(a.replication_bound, None);
}

#[test]
fn typed_values_are_parsed() {
    let a = run_ok(&[
        "file.spthy",
        "--bound=5",
        "-c3",
        "--saturation=7",
        "--stop-on-trace=BFS",
        "--output-module=deepsec",
        "--partial-evaluation=Verbose",
        "--replication-bound=-2",
        "--diff",
        "--parse-only",
    ]);
    assert_eq!(a.bound, Some(5));
    assert_eq!(a.open_chains, 3);
    assert_eq!(a.saturation, 7);
    assert_eq!(a.stop_on_trace, StopOnTrace::Bfs);
    assert_eq!(a.output_module, Some(OutputModule::Deepsec));
    assert_eq!(a.partial_evaluation, Some(PartialEval::Verbose));
    assert_eq!(a.replication_bound, Some(-2));
    assert!(a.diff);
    assert!(a.parse_only);
    assert_eq!(a.positional, vec![s("file.spthy")]);
}

// ---- GAP 1: consumer-extension flags ------------------------------------------

#[test]
fn interop_flags_absent_from_every_help_page() {
    for mode in [Mode::Batch, Mode::Interactive, Mode::Variants, Mode::Test] {
        let h = render_help(mode);
        assert!(!h.contains("processors"), "{mode:?} help must not mention processors");
        assert!(!h.contains("maude-processes"), "{mode:?} help must not mention maude-processes");
        assert!(!h.contains("data-dir"), "{mode:?} help must not mention data-dir");
    }
}

#[test]
fn interop_flags_typed_both_forms() {
    // '=' form.
    let a = run_ok(&["file.spthy", "--processors=4", "--maude-processes=2", "--data-dir=/tmp/d"]);
    assert_eq!(a.processors, Some(4));
    assert_eq!(a.maude_processes, Some(2));
    assert_eq!(a.data_dir, Some(s("/tmp/d")));
    // Space-separated form consumes the following token (consumer surface).
    let b = run_ok(&["file.spthy", "--processors", "8"]);
    assert_eq!(b.processors, Some(8));
    assert_eq!(b.positional, vec![s("file.spthy")]);
    // Non-positive / non-integer rejected (original, non-reference text).
    assert!(parse_args(&argv(&["file.spthy", "--processors=0"])).is_err());
    assert!(parse_args(&argv(&["file.spthy", "--processors=x"])).is_err());
    // Bare form with no following token -> missing-value error.
    assert!(parse_args(&argv(&["--processors"])).is_err());
}

// ---- GAP 2: stream-aware batch framing ----------------------------------------

/// Extract each theory block's dynamic slots `(analyzed, output, time)` from a
/// summary section, so the framing tests are robust to path/timing variation.
fn slots(section: &str) -> Vec<(String, Option<String>, f64)> {
    let mut out: Vec<(String, Option<String>, f64)> = Vec::new();
    let mut cur: Option<(String, Option<String>, f64)> = None;
    for line in section.lines() {
        if let Some(a) = line.strip_prefix("analyzed: ") {
            if let Some(c) = cur.take() {
                out.push(c);
            }
            cur = Some((a.to_string(), None, 0.0));
        } else if let Some(o) = line.strip_prefix("  output:") {
            if let Some(c) = cur.as_mut() {
                c.1 = Some(o.trim_start().to_string());
            }
        } else if let Some(t) = line.strip_prefix("  processing time: ") {
            if let Some(c) = cur.as_mut() {
                c.2 = t.trim_end_matches('s').parse().expect("time");
            }
        }
    }
    if let Some(c) = cur.take() {
        out.push(c);
    }
    out
}

fn summary_start(out: &str) -> usize {
    let sep = format!("\n{}", "=".repeat(78));
    out.find(&sep).expect("summary rule")
}

fn lemma(name: &str, kind: TraceKind, result: LemmaResult, steps: u64) -> LemmaOutcome {
    LemmaOutcome::whole(s(name), kind, result, steps)
}

fn nslpk3_entries(nonce_result: LemmaResult, nonce_steps: u64) -> Vec<LemmaOutcome> {
    vec![
        lemma("types", TraceKind::AllTraces, LemmaResult::AnalysisIncomplete, 1),
        lemma("nonce_secrecy", TraceKind::AllTraces, nonce_result, nonce_steps),
        lemma("injective_agree", TraceKind::AllTraces, LemmaResult::AnalysisIncomplete, 1),
        lemma("session_key_setup_possible", TraceKind::ExistsTrace, LemmaResult::AnalysisIncomplete, 1),
    ]
}

#[test]
fn frame_batch_default_reproduces_both_streams() {
    let out_cap = include_str!("fixtures/split_batch_default.out.txt");
    let err_cap = include_str!("fixtures/split_batch_default.err.txt");
    let cut = summary_start(out_cap);
    let payload = &out_cap[..cut];
    let (analyzed, output, time) = slots(&out_cap[cut..]).remove(0);
    let theory = BatchTheory {
        name: s("NSLPK3"),
        payload: Some(payload.to_string()),
        extra_progress: String::new(),
        summary: Summary {
            analyzed,
            output,
            processing_time: time,
            warnings: None,
            lemmas: nslpk3_entries(LemmaResult::AnalysisIncomplete, 1),
        },
    };
    let streams = frame_batch(&MaudeInfo::default(), &[theory]);
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

#[test]
fn frame_batch_prove_reproduces_both_streams() {
    let out_cap = include_str!("fixtures/split_batch_prove.out.txt");
    let err_cap = include_str!("fixtures/split_batch_prove.err.txt");
    let cut = summary_start(out_cap);
    let (analyzed, output, time) = slots(&out_cap[cut..]).remove(0);
    // Under --prove the engine emits extra stderr progress after "Theory closed".
    let marker = "[Theory NSLPK3] Theory closed\n";
    let closed_end = err_cap.find(marker).unwrap() + marker.len();
    let extra_progress = err_cap[closed_end..].to_string();
    assert!(extra_progress.contains("[Saturating Sources]"));
    let theory = BatchTheory {
        name: s("NSLPK3"),
        payload: Some(out_cap[..cut].to_string()),
        extra_progress,
        summary: Summary {
            analyzed,
            output,
            processing_time: time,
            warnings: None,
            lemmas: nslpk3_entries(LemmaResult::Verified, 54),
        },
    };
    let streams = frame_batch(&MaudeInfo::default(), &[theory]);
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

#[test]
fn frame_batch_output_file_summary_has_aligned_output_line() {
    let out_cap = include_str!("fixtures/split_batch_output_single.out.txt");
    let err_cap = include_str!("fixtures/split_batch_output_single.err.txt");
    // No payload printed: everything written to a file; stdout is the summary only.
    let (analyzed, output, time) = slots(out_cap).remove(0);
    assert!(output.is_some(), "an output path must be present");
    let theory = BatchTheory {
        name: s("NSLPK3"),
        payload: None,
        extra_progress: String::new(),
        summary: Summary {
            analyzed,
            output,
            processing_time: time,
            warnings: None,
            lemmas: nslpk3_entries(LemmaResult::AnalysisIncomplete, 1),
        },
    };
    let streams = frame_batch(&MaudeInfo::default(), &[theory]);
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

#[test]
fn frame_batch_multifile_reproduces_both_streams() {
    let out_cap = include_str!("fixtures/split_batch_multifile.out.txt");
    let err_cap = include_str!("fixtures/split_batch_multifile.err.txt");
    let cut = summary_start(out_cap);
    let payload_region = &out_cap[..cut];
    let split = payload_region.find("theory FreshPubConst").expect("second theory");
    let blocks = slots(&out_cap[cut..]);
    let t1 = BatchTheory {
        name: s("NSLPK3"),
        payload: Some(payload_region[..split].to_string()),
        extra_progress: String::new(),
        summary: Summary {
            analyzed: blocks[0].0.clone(),
            output: blocks[0].1.clone(),
            processing_time: blocks[0].2,
            warnings: None,
            lemmas: nslpk3_entries(LemmaResult::AnalysisIncomplete, 1),
        },
    };
    let t2 = BatchTheory {
        name: s("FreshPubConst"),
        payload: Some(payload_region[split..].to_string()),
        extra_progress: String::new(),
        summary: Summary {
            analyzed: blocks[1].0.clone(),
            output: blocks[1].1.clone(),
            processing_time: blocks[1].2,
            // Default-mode warning theory: count line only, no advisory line.
            warnings: Some(WarningSummary { failed_checks: 1, analysis_maybe_wrong: false }),
            lemmas: vec![],
        },
    };
    let streams = frame_batch(&MaudeInfo::default(), &[t1, t2]);
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

#[test]
fn frame_parse_only_single_and_multi() {
    // Single theory.
    let out1 = include_str!("fixtures/split_batch_parseonly.out.txt");
    let err1 = include_str!("fixtures/split_batch_parseonly.err.txt");
    let s1 = frame_parse_only(&[LoadedTheory { name: s("NSLPK3"), payload: out1.to_string() }]);
    assert_eq!(s1.out, out1);
    assert_eq!(s1.err, err1);

    // Two theories: payloads concatenate directly; one `Theory loaded` per theory.
    let out2 = include_str!("fixtures/split_parseonly_multi.out.txt");
    let err2 = include_str!("fixtures/split_parseonly_multi.err.txt");
    let split = out2.find("theory FreshPubConst").expect("second theory");
    let s2 = frame_parse_only(&[
        LoadedTheory { name: s("NSLPK3"), payload: out2[..split].to_string() },
        LoadedTheory { name: s("FreshPubConst"), payload: out2[split..].to_string() },
    ]);
    assert_eq!(s2.out, out2);
    assert_eq!(s2.err, err2);
}

#[test]
fn frame_variants_splits_payload_and_preamble() {
    let out_cap = include_str!("fixtures/split_variants.out.txt");
    let err_cap = include_str!("fixtures/split_variants.err.txt");
    let streams = frame_variants(&MaudeInfo::default(), out_cap);
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

#[test]
fn summary_output_column_alignment_is_exact() {
    // Value column is 19 (0-indexed): labels padded to the width of
    // "processing time:" (16) + one space. "output:" therefore gets 10 spaces.
    let summary = Summary {
        analyzed: s("T.spthy"),
        output: Some(s("/out/T.spthy")),
        processing_time: 0.4,
        warnings: None,
        lemmas: vec![],
    };
    let block = render_summary(std::slice::from_ref(&summary));
    let output_line = format!("  output:{}{}", " ".repeat(10), "/out/T.spthy");
    assert!(block.contains(&format!("{}\n", output_line)), "output line: {block:?}");
    assert!(block.contains("  processing time: 0.40s\n"));
    // Both values start at column 19.
    for line in block.lines() {
        if let Some(rest) = line.strip_prefix("  output:") {
            assert_eq!(2 + "output:".len() + (rest.len() - rest.trim_start().len()), 19);
        }
    }
}

// ---- GAP 1 (Round 5): summary content — warnings + lemmas, verdicts, bounded ---

/// Frame a single closed theory from its captured `.out`/`.err` fixture: slice the
/// opaque payload and summary slots out of stdout, recover the theory's
/// `extra_progress` (anything after its `Theory closed` marker) from stderr, and
/// reassemble both streams with the given summary content.
fn frame_single(
    out_cap: &str,
    err_cap: &str,
    name: &str,
    warnings: Option<WarningSummary>,
    lemmas: Vec<LemmaOutcome>,
) -> Streams {
    let cut = summary_start(out_cap);
    let (analyzed, output, time) = slots(&out_cap[cut..]).remove(0);
    let marker = format!("[Theory {name}] Theory closed\n");
    let closed_end = err_cap.find(&marker).expect("closed marker") + marker.len();
    let theory = BatchTheory {
        name: s(name),
        payload: Some(out_cap[..cut].to_string()),
        extra_progress: err_cap[closed_end..].to_string(),
        summary: Summary { analyzed, output, processing_time: time, warnings, lemmas },
    };
    frame_batch(&MaudeInfo::default(), &[theory])
}

fn warn(failed_checks: u64, analysis_maybe_wrong: bool) -> Option<WarningSummary> {
    Some(WarningSummary { failed_checks, analysis_maybe_wrong })
}

#[test]
fn summary_warning_and_lemma_under_prove() {
    // A single theory with BOTH a wellformedness warning and a proved lemma: the
    // warning heading carries the advisory second line (proving run), a `  ` line
    // separates the warning heading from the lemma line, and the lemma is verified.
    let out_cap = include_str!("fixtures/r5_warn_lemma_prove.out.txt");
    let err_cap = include_str!("fixtures/r5_warn_lemma_prove.err.txt");
    let streams = frame_single(
        out_cap,
        err_cap,
        "WarnAndLemma",
        warn(1, true),
        vec![lemma("trivial_true", TraceKind::AllTraces, LemmaResult::Verified, 2)],
    );
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

#[test]
fn summary_warning_and_lemma_default_has_no_advisory_line() {
    // Same theory, plain analyze: the warning heading is the count line only (no
    // advisory), still separated from the lemma line by a `  ` line; the lemma is
    // analysis-incomplete.
    let out_cap = include_str!("fixtures/r5_warn_lemma_default.out.txt");
    let err_cap = include_str!("fixtures/r5_warn_lemma_default.err.txt");
    let streams = frame_single(
        out_cap,
        err_cap,
        "WarnAndLemma",
        warn(1, false),
        vec![lemma("trivial_true", TraceKind::AllTraces, LemmaResult::AnalysisIncomplete, 1)],
    );
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

#[test]
fn summary_falsified_wording_depends_on_trace_kind() {
    // all-traces falsified -> "falsified - found trace"; exists-trace falsified ->
    // "falsified - no trace found". Warning + advisory precede the lemma lines.
    let out_cap = include_str!("fixtures/r5_falsify_prove.out.txt");
    let err_cap = include_str!("fixtures/r5_falsify_prove.err.txt");
    let streams = frame_single(
        out_cap,
        err_cap,
        "FalsifyMe",
        warn(1, true),
        vec![
            lemma("secrecy_all", TraceKind::AllTraces, LemmaResult::Falsified, 3),
            lemma("never_happens", TraceKind::ExistsTrace, LemmaResult::Falsified, 2),
        ],
    );
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
    // Lock the exact verdict phrases in the observed bytes.
    assert!(streams.out.contains("secrecy_all (all-traces): falsified - found trace (3 steps)\n"));
    assert!(streams
        .out
        .contains("never_happens (exists-trace): falsified - no trace found (2 steps)\n"));
}

#[test]
fn summary_warning_no_lemma_under_prove_has_advisory_no_separator() {
    // Warning, no lemmas, proving run: count line + advisory line, and NO trailing
    // `  ` separator (there is no lemma section to separate from).
    let out_cap = include_str!("fixtures/r5_freshpub_prove.out.txt");
    let err_cap = include_str!("fixtures/r5_freshpub_prove.err.txt");
    let streams = frame_single(out_cap, err_cap, "FreshPubConst", warn(1, true), vec![]);
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

#[test]
fn summary_no_warning_no_lemma_is_just_the_opening_line() {
    // A well-formed theory with no lemmas: the body is only the opening `  ` line.
    let out_cap = include_str!("fixtures/r5_nolemma_default.out.txt");
    let err_cap = include_str!("fixtures/r5_nolemma_default.err.txt");
    let streams = frame_single(out_cap, err_cap, "NoLemmaClean", None, vec![]);
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

#[test]
fn summary_verified_wording_uniform_across_trace_kinds() {
    // Both an exists-trace and an all-traces verified lemma print plain "verified".
    let out_cap = include_str!("fixtures/r5_exists_verified.out.txt");
    let err_cap = include_str!("fixtures/r5_exists_verified.err.txt");
    let streams = frame_single(
        out_cap,
        err_cap,
        "ExistsVerified",
        None,
        vec![
            lemma("can_ping", TraceKind::ExistsTrace, LemmaResult::Verified, 2),
            lemma("always_true", TraceKind::AllTraces, LemmaResult::Verified, 2),
        ],
    );
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

#[test]
fn summary_bounded_prove_yields_only_incomplete_lines() {
    // GAP 1(a): a proving run cut short by `--bound` adds no extra summary line —
    // every unclosed lemma is a plain "analysis incomplete (<explored> steps)",
    // and with no warning there is no advisory line. Only the step counts differ.
    let out_cap = include_str!("fixtures/r5_nslpk3_prove_bound2.out.txt");
    let err_cap = include_str!("fixtures/r5_nslpk3_prove_bound2.err.txt");
    let streams = frame_single(
        out_cap,
        err_cap,
        "NSLPK3",
        None,
        vec![
            lemma("types", TraceKind::AllTraces, LemmaResult::AnalysisIncomplete, 1),
            lemma("nonce_secrecy", TraceKind::AllTraces, LemmaResult::AnalysisIncomplete, 6),
            lemma("injective_agree", TraceKind::AllTraces, LemmaResult::AnalysisIncomplete, 1),
            lemma("session_key_setup_possible", TraceKind::ExistsTrace, LemmaResult::AnalysisIncomplete, 1),
        ],
    );
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

#[test]
fn summary_multifile_lemmas_then_warn_and_lemma_under_prove() {
    // GAP 1(b) between-theory joining: a lemmas-only theory followed by a
    // warn+lemma theory, one combined summary. Each theory block is preceded by a
    // blank line; the warn theory carries advisory + `  ` separator internally.
    let out_cap = include_str!("fixtures/r5_multi_warn_prove.out.txt");
    let err_cap = include_str!("fixtures/r5_multi_warn_prove.err.txt");
    let cut = summary_start(out_cap);
    let payload_region = &out_cap[..cut];
    let split = payload_region.find("theory WarnAndLemma").expect("second theory");
    let blocks = slots(&out_cap[cut..]);
    // NSLPK3's extra progress is everything on stderr between its `Theory closed`
    // marker and the start of the next theory's progress block.
    let m1 = "[Theory NSLPK3] Theory closed\n";
    let m2 = "[Theory WarnAndLemma] Theory loaded\n";
    let s1 = err_cap.find(m1).unwrap() + m1.len();
    let s2 = err_cap.find(m2).unwrap();
    let extra1 = err_cap[s1..s2].to_string();
    let t1 = BatchTheory {
        name: s("NSLPK3"),
        payload: Some(payload_region[..split].to_string()),
        extra_progress: extra1,
        summary: Summary {
            analyzed: blocks[0].0.clone(),
            output: blocks[0].1.clone(),
            processing_time: blocks[0].2,
            warnings: None,
            lemmas: vec![
                lemma("types", TraceKind::AllTraces, LemmaResult::Verified, 32),
                lemma("nonce_secrecy", TraceKind::AllTraces, LemmaResult::Verified, 54),
                lemma("injective_agree", TraceKind::AllTraces, LemmaResult::Verified, 92),
                lemma("session_key_setup_possible", TraceKind::ExistsTrace, LemmaResult::Verified, 5),
            ],
        },
    };
    let t2 = BatchTheory {
        name: s("WarnAndLemma"),
        payload: Some(payload_region[split..].to_string()),
        extra_progress: String::new(),
        summary: Summary {
            analyzed: blocks[1].0.clone(),
            output: blocks[1].1.clone(),
            processing_time: blocks[1].2,
            warnings: warn(1, true),
            lemmas: vec![lemma("trivial_true", TraceKind::AllTraces, LemmaResult::Verified, 2)],
        },
    };
    let streams = frame_batch(&MaudeInfo::default(), &[t1, t2]);
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

#[test]
fn summary_section_bytes_are_exact() {
    // Pin the exact body bytes for the warn+lemma+prove shape independent of any
    // captured payload: opening `  ` line, warning count line, advisory line
    // (11-space aligned), `  ` separator, then the lemma line.
    let summary = Summary {
        analyzed: s("T.spthy"),
        output: None,
        processing_time: 0.06,
        warnings: warn(1, true),
        lemmas: vec![lemma("trivial_true", TraceKind::AllTraces, LemmaResult::Verified, 2)],
    };
    let block = render_summary(std::slice::from_ref(&summary));
    let expected_body = concat!(
        "  processing time: 0.06s\n",
        "  \n",
        "  WARNING: 1 wellformedness check failed!\n",
        "           The analysis results might be wrong!\n",
        "  \n",
        "  trivial_true (all-traces): verified (2 steps)\n",
    );
    assert!(block.contains(expected_body), "block was: {block:?}");
    // Advisory line aligns exactly under the text after "  WARNING: " (11 cols).
    for line in block.lines() {
        if line.trim_start().starts_with("The analysis results") {
            assert_eq!(line.len() - line.trim_start().len(), "  WARNING: ".len());
        }
    }
}

// ---- GAP 2 (Round 5): incremental (streaming) emission ------------------------

/// Drive the incremental emitter over a `&[BatchTheory]` and return the per-stream
/// buffers it accumulated.
fn streamed(theories: &[BatchTheory]) -> Streams {
    let mut sink = StreamCollector::default();
    drive_batch(&mut sink, &MaudeInfo::default(), theories);
    sink.streams
}

fn nslpk3_theory(result: LemmaResult, nonce_steps: u64, payload: &str) -> BatchTheory {
    BatchTheory {
        name: s("NSLPK3"),
        payload: Some(payload.to_string()),
        extra_progress: String::new(),
        summary: Summary {
            analyzed: s("classic/NSLPK3.spthy"),
            output: None,
            processing_time: 0.4,
            warnings: None,
            lemmas: nslpk3_entries(result, nonce_steps),
        },
    }
}

#[test]
fn streaming_matches_assembled_on_shared_inputs() {
    // The interop equivalence: for the same inputs, the incremental emitter's
    // per-stream bytes equal the assembled frame_batch model's.
    let warn_theory = BatchTheory {
        name: s("WarnAndLemma"),
        payload: Some(s("theory WarnAndLemma\n...opaque payload...\nend\n")),
        extra_progress: String::new(),
        summary: Summary {
            analyzed: s("warn_and_lemma.spthy"),
            output: None,
            processing_time: 0.06,
            warnings: warn(1, true),
            lemmas: vec![lemma("trivial_true", TraceKind::AllTraces, LemmaResult::Verified, 2)],
        },
    };
    let cases: Vec<Vec<BatchTheory>> = vec![
        vec![],
        vec![nslpk3_theory(LemmaResult::AnalysisIncomplete, 1, "theory NSLPK3\n..payload..\nend\n")],
        vec![BatchTheory {
            extra_progress: s("[Saturating Sources] Step 1 (Max 5)\n[Saturating Sources] Done\n"),
            ..nslpk3_theory(LemmaResult::Verified, 54, "theory NSLPK3\n..payload..\nend\n")
        }],
        vec![
            nslpk3_theory(LemmaResult::Verified, 54, "theory NSLPK3\n..p1..\nend\n"),
            warn_theory.clone(),
        ],
        // A theory written to an output file emits no payload, only its summary.
        vec![BatchTheory {
            payload: None,
            summary: Summary {
                output: Some(s("/out/NSLPK3.spthy")),
                ..nslpk3_theory(LemmaResult::AnalysisIncomplete, 1, "").summary
            },
            ..nslpk3_theory(LemmaResult::AnalysisIncomplete, 1, "")
        }],
    ];
    for theories in &cases {
        let assembled = frame_batch(&MaudeInfo::default(), theories);
        let streamed = streamed(theories);
        assert_eq!(streamed.out, assembled.out, "stdout mismatch for {theories:?}");
        assert_eq!(streamed.err, assembled.err, "stderr mismatch for {theories:?}");
    }
}

#[test]
fn streaming_reproduces_captured_multifile_streams() {
    // Drive the emitter from the real multi-theory capture and check byte parity
    // against the fixture on both streams (streaming path, end to end).
    let out_cap = include_str!("fixtures/r5_multi_warn_prove.out.txt");
    let err_cap = include_str!("fixtures/r5_multi_warn_prove.err.txt");
    let cut = summary_start(out_cap);
    let payload_region = &out_cap[..cut];
    let split = payload_region.find("theory WarnAndLemma").expect("second theory");
    let blocks = slots(&out_cap[cut..]);
    let m1 = "[Theory NSLPK3] Theory closed\n";
    let m2 = "[Theory WarnAndLemma] Theory loaded\n";
    let s1 = err_cap.find(m1).unwrap() + m1.len();
    let s2 = err_cap.find(m2).unwrap();
    let t1 = BatchTheory {
        name: s("NSLPK3"),
        payload: Some(payload_region[..split].to_string()),
        extra_progress: err_cap[s1..s2].to_string(),
        summary: Summary {
            analyzed: blocks[0].0.clone(),
            output: blocks[0].1.clone(),
            processing_time: blocks[0].2,
            warnings: None,
            lemmas: vec![
                lemma("types", TraceKind::AllTraces, LemmaResult::Verified, 32),
                lemma("nonce_secrecy", TraceKind::AllTraces, LemmaResult::Verified, 54),
                lemma("injective_agree", TraceKind::AllTraces, LemmaResult::Verified, 92),
                lemma("session_key_setup_possible", TraceKind::ExistsTrace, LemmaResult::Verified, 5),
            ],
        },
    };
    let t2 = BatchTheory {
        name: s("WarnAndLemma"),
        payload: Some(payload_region[split..].to_string()),
        extra_progress: String::new(),
        summary: Summary {
            analyzed: blocks[1].0.clone(),
            output: blocks[1].1.clone(),
            processing_time: blocks[1].2,
            warnings: warn(1, true),
            lemmas: vec![lemma("trivial_true", TraceKind::AllTraces, LemmaResult::Verified, 2)],
        },
    };
    let streams = streamed(&[t1, t2]);
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

#[test]
fn streaming_manual_drive_orders_streams_correctly() {
    // The emitter, driven by hand event-by-event (as a real consumer would while
    // the prover runs), yields the same per-stream bytes as frame_batch. This is
    // the "consumer chooses flush timing" surface — the collector is one Sink;
    // ordering and content are the emitter's contract.
    let theory = BatchTheory {
        name: s("NSLPK3"),
        payload: Some(s("theory NSLPK3\n..payload..\nend\n")),
        extra_progress: s("[Saturating Sources] Step 1 (Max 5)\n[Saturating Sources] Done\n"),
        summary: Summary {
            analyzed: s("classic/NSLPK3.spthy"),
            output: None,
            processing_time: 0.4,
            warnings: None,
            lemmas: nslpk3_entries(LemmaResult::Verified, 54),
        },
    };
    let mut sink = StreamCollector::default();
    {
        let mut e = BatchEmitter::begin(&mut sink, &MaudeInfo::default());
        e.closed_phases("NSLPK3");
        e.extra_progress("[Saturating Sources] Step 1 (Max 5)\n[Saturating Sources] Done\n");
        e.payload(Some("theory NSLPK3\n..payload..\nend\n"));
        e.record_summary(theory.summary.clone());
        e.finish();
    }
    let assembled = frame_batch(&MaudeInfo::default(), std::slice::from_ref(&theory));
    assert_eq!(sink.streams.out, assembled.out);
    assert_eq!(sink.streams.err, assembled.err);
}

// ---- runtime error renderers --------------------------------------------------

fn last_line(text: &str) -> &str {
    text.trim_end_matches('\n').rsplit('\n').next().unwrap()
}

#[test]
fn open_file_error_lines_match() {
    let missing = include_str!("fixtures/err_missing_file.txt");
    assert_eq!(
        open_file_error("/nonexistent/xyz.spthy", OpenFileError::DoesNotExist).trim_end(),
        last_line(missing)
    );
    let unreadable = include_str!("fixtures/err_unreadable.txt");
    let ur_path = last_line(unreadable)
        .strip_prefix("tamarin-prover: ")
        .unwrap()
        .split(": openFile")
        .next()
        .unwrap();
    assert_eq!(
        open_file_error(ur_path, OpenFileError::PermissionDenied).trim_end(),
        last_line(unreadable)
    );
    let isdir = include_str!("fixtures/err_isdir.txt");
    let dir_path = last_line(isdir)
        .strip_prefix("tamarin-prover: ")
        .unwrap()
        .split(": openFile")
        .next()
        .unwrap();
    assert_eq!(open_file_error(dir_path, OpenFileError::IsDirectory).trim_end(), last_line(isdir));
}

#[test]
fn app_error_callstack_matches() {
    let bound = include_str!("fixtures/err_bound_nonint.txt");
    assert!(bound.ends_with(&app_error("bound: invalid bound given")));
    let outmod = include_str!("fixtures/err_output_module_bad.txt");
    assert!(outmod.ends_with(&app_error("output mode not supported.")));
}

// ---- structural parse semantics (still exercised) -----------------------------

#[test]
fn structural_parse_still_records_flags() {
    match parse(&argv(&["file.spthy", "--bound", "5"])).unwrap() {
        Command::Run(spec) => {
            assert!(spec.options.is_set("bound"));
            // Space-separated value is NOT consumed by a reference flag: 5 is a file.
            assert_eq!(spec.positional, vec![s("file.spthy"), s("5")]);
        }
        other => panic!("expected Run, got {other:?}"),
    }
}

// ---- Round 6: --diff summary line taxonomy (RHS / LHS / DiffLemma) -------------
//
// Under `--diff` the summary body gains prefixed line forms (captures r6_diff_*):
//   `  RHS :  <name> (<kind>): <verdict> (<N> steps)`   projected ordinary lemma
//   `  LHS :  <name> (<kind>): <verdict> (<N> steps)`   projected ordinary lemma
//   `  DiffLemma:  <name> : <verdict> (<N> steps)`      observational equivalence
// The verdict phrases are the same set as non-diff lemmas; a DiffLemma has no
// trace-kind and its `falsified` always reads `- found trace`.

fn rhs(name: &str, kind: TraceKind, result: LemmaResult, steps: u64) -> LemmaOutcome {
    LemmaOutcome::projected(LemmaSide::Rhs, s(name), kind, result, steps)
}
fn lhs(name: &str, kind: TraceKind, result: LemmaResult, steps: u64) -> LemmaOutcome {
    LemmaOutcome::projected(LemmaSide::Lhs, s(name), kind, result, steps)
}
fn diff_lemma(name: &str, result: LemmaResult, steps: u64) -> LemmaOutcome {
    LemmaOutcome::diff_lemma(s(name), result, steps)
}

/// Build a single-theory summary from a `--diff` capture's dynamic slots and
/// assert `render_summary` reproduces that capture's summary section byte-exactly.
fn assert_diff_summary(out_cap: &str, warnings: Option<WarningSummary>, lemmas: Vec<LemmaOutcome>) {
    let section = &out_cap[summary_start(out_cap)..];
    let (analyzed, output, time) = slots(section).remove(0);
    let summary = Summary { analyzed, output, processing_time: time, warnings, lemmas };
    assert_eq!(render_summary(std::slice::from_ref(&summary)), section);
}

#[test]
fn diff_summary_regular_lemma_then_obs_equiv_verified() {
    // --diff --prove: one ordinary lemma projected to RHS then LHS, then the
    // observational-equivalence DiffLemma (verified, no trace-kind).
    let out_cap = include_str!("fixtures/r6_diff_with_lemma.out.txt");
    assert_diff_summary(
        out_cap,
        None,
        vec![
            rhs("sent_secret", TraceKind::AllTraces, LemmaResult::Verified, 3),
            lhs("sent_secret", TraceKind::AllTraces, LemmaResult::Verified, 3),
            diff_lemma("Observational_equivalence", LemmaResult::Verified, 50),
        ],
    );
}

#[test]
fn diff_summary_two_lemmas_grouped_rhs_then_lhs_per_lemma() {
    // Two ordinary lemmas: RHS/LHS are grouped PER lemma (RHS l1, LHS l1, RHS l2,
    // LHS l2), the exists-trace kind renders normally, then the trailing DiffLemma.
    let out_cap = include_str!("fixtures/r6_diff_two_lemmas.out.txt");
    assert_diff_summary(
        out_cap,
        None,
        vec![
            rhs("sent_secret", TraceKind::AllTraces, LemmaResult::Verified, 3),
            lhs("sent_secret", TraceKind::AllTraces, LemmaResult::Verified, 3),
            rhs("can_send", TraceKind::ExistsTrace, LemmaResult::Verified, 2),
            lhs("can_send", TraceKind::ExistsTrace, LemmaResult::Verified, 2),
            diff_lemma("Observational_equivalence", LemmaResult::Verified, 50),
        ],
    );
}

#[test]
fn diff_summary_warning_precedes_diff_lines_under_prove() {
    // --diff --prove with a wellformedness warning: the WARNING section (with the
    // --prove advisory line) comes first, then a `  ` separator, then the RHS/LHS
    // lines and a FALSIFIED DiffLemma (`- found trace`).
    let out_cap = include_str!("fixtures/r6_diff_warn.out.txt");
    assert_diff_summary(
        out_cap,
        Some(WarningSummary { failed_checks: 1, analysis_maybe_wrong: true }),
        vec![
            rhs("happens", TraceKind::AllTraces, LemmaResult::Verified, 2),
            lhs("happens", TraceKind::AllTraces, LemmaResult::Verified, 2),
            diff_lemma("Observational_equivalence", LemmaResult::Falsified, 7),
        ],
    );
}

#[test]
fn diff_summary_obs_equiv_incomplete_only() {
    // --diff without --prove: the DiffLemma alone, analysis incomplete.
    let out_cap = include_str!("fixtures/r6_diff_default.out.txt");
    assert_diff_summary(
        out_cap,
        None,
        vec![diff_lemma("Observational_equivalence", LemmaResult::AnalysisIncomplete, 1)],
    );
}

#[test]
fn diff_summary_obs_equiv_falsified_found_trace() {
    // --diff --prove: the DiffLemma falsified reads `- found trace` (a
    // distinguishing attack was found).
    let out_cap = include_str!("fixtures/r6_diff_n5n6.out.txt");
    assert_diff_summary(
        out_cap,
        None,
        vec![diff_lemma("Observational_equivalence", LemmaResult::Falsified, 8)],
    );
}

#[test]
fn diff_summary_line_bytes_are_exact() {
    // Pin the exact diff line prefixes independent of any capture, using only
    // observed forms: `RHS :  ` / `LHS :  ` before a normal lemma line (each kind),
    // and `DiffLemma:  <name> : <verdict>` (no trace-kind) for obs-equivalence.
    let summary = Summary {
        analyzed: s("D.spthy"),
        output: None,
        processing_time: 0.10,
        warnings: None,
        lemmas: vec![
            rhs("l", TraceKind::AllTraces, LemmaResult::Verified, 3),
            lhs("l", TraceKind::ExistsTrace, LemmaResult::Verified, 2),
            diff_lemma("Observational_equivalence", LemmaResult::Falsified, 8),
        ],
    };
    let block = render_summary(std::slice::from_ref(&summary));
    let expected_body = concat!(
        "  processing time: 0.10s\n",
        "  \n",
        "  RHS :  l (all-traces): verified (3 steps)\n",
        "  LHS :  l (exists-trace): verified (2 steps)\n",
        "  DiffLemma:  Observational_equivalence : falsified - found trace (8 steps)\n",
    );
    assert!(block.contains(expected_body), "block was: {block:?}");
}

// ---- Round 6 (cont.): projected RHS/LHS non-verified verdicts ------------------
//
// Every earlier diff capture showed the projected RHS/LHS lines only as
// `verified`. These fixtures drive the projected lines into their remaining
// terminal states, proving the projected form carries the SAME kind-dependent
// verdict phrases as a whole-theory lemma and that the two sides are computed
// independently (not mirrored).

#[test]
fn diff_summary_projected_falsified_both_kinds() {
    // --diff --prove where both projected systems falsify the ordinary lemmas: an
    // all-traces lemma reads `falsified - found trace`, an exists-trace lemma reads
    // `falsified - no trace found`, on BOTH the RHS and LHS lines. (The
    // obs-equivalence DiffLemma is independently verified.)
    let out_cap = include_str!("fixtures/r6_diff_false.out.txt");
    assert_diff_summary(
        out_cap,
        None,
        vec![
            rhs("leaked", TraceKind::AllTraces, LemmaResult::Falsified, 3),
            lhs("leaked", TraceKind::AllTraces, LemmaResult::Falsified, 3),
            rhs("impossible", TraceKind::ExistsTrace, LemmaResult::Falsified, 2),
            lhs("impossible", TraceKind::ExistsTrace, LemmaResult::Falsified, 2),
            diff_lemma("Observational_equivalence", LemmaResult::Verified, 67),
        ],
    );
}

#[test]
fn diff_summary_projected_incomplete_without_prove() {
    // --diff without --prove: the projected RHS/LHS lines of an ordinary lemma read
    // `analysis incomplete`, exactly like the trailing DiffLemma.
    let out_cap = include_str!("fixtures/r6_diff_lemma_noprove.out.txt");
    assert_diff_summary(
        out_cap,
        None,
        vec![
            rhs("sent_secret", TraceKind::AllTraces, LemmaResult::AnalysisIncomplete, 1),
            lhs("sent_secret", TraceKind::AllTraces, LemmaResult::AnalysisIncomplete, 1),
            diff_lemma("Observational_equivalence", LemmaResult::AnalysisIncomplete, 1),
        ],
    );
}

#[test]
fn diff_summary_projected_sides_are_independent() {
    // --diff --prove of a theory whose two projected systems genuinely differ: the
    // ordinary lemma is verified on the RHS but falsified on the LHS, so the two
    // projected lines carry DIFFERENT verdicts. Confirms the sides are not mirrored.
    let out_cap = include_str!("fixtures/r6_diff_asym.out.txt");
    assert_diff_summary(
        out_cap,
        None,
        vec![
            rhs("secret", TraceKind::AllTraces, LemmaResult::Verified, 3),
            lhs("secret", TraceKind::AllTraces, LemmaResult::Falsified, 3),
            diff_lemma("Observational_equivalence", LemmaResult::Verified, 44),
        ],
    );
}

#[test]
fn frame_batch_multi_warn_then_two_lemmas_reproduces_both_streams() {
    // Re-verify per-lemma / warning-block interleaving in a multi-lemma,
    // multi-theory run: theory 1 is a warn+lemma theory (warning section with the
    // --prove advisory, `  ` separator, then its single lemma line); theory 2
    // carries two lemmas in declaration order. Between-theory join is a blank line
    // before each `analyzed:`; stderr is the preamble once then each theory's five
    // phases (neither tiny theory emits saturating-sources progress).
    let out_cap = include_str!("fixtures/r6_multi_warn_two.out.txt");
    let err_cap = include_str!("fixtures/r6_multi_warn_two.err.txt");
    let cut = summary_start(out_cap);
    let payload_region = &out_cap[..cut];
    let split = payload_region.find("theory TwoLemma").expect("second theory");
    let blocks = slots(&out_cap[cut..]);
    let t1 = BatchTheory {
        name: s("WarnAndLemma"),
        payload: Some(payload_region[..split].to_string()),
        extra_progress: String::new(),
        summary: Summary {
            analyzed: blocks[0].0.clone(),
            output: blocks[0].1.clone(),
            processing_time: blocks[0].2,
            warnings: warn(1, true),
            lemmas: vec![lemma("trivial_true", TraceKind::AllTraces, LemmaResult::Verified, 2)],
        },
    };
    let t2 = BatchTheory {
        name: s("TwoLemma"),
        payload: Some(payload_region[split..].to_string()),
        extra_progress: String::new(),
        summary: Summary {
            analyzed: blocks[1].0.clone(),
            output: blocks[1].1.clone(),
            processing_time: blocks[1].2,
            warnings: None,
            lemmas: vec![
                lemma("first", TraceKind::AllTraces, LemmaResult::Verified, 2),
                lemma("second", TraceKind::ExistsTrace, LemmaResult::Verified, 2),
            ],
        },
    };
    let streams = frame_batch(&MaudeInfo::default(), &[t1, t2]);
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

// ---- Round 7: the non-diff "analysis cannot be finished" verdict ---------------
//
// A `--prove` run of a theory whose proof reaches a terminal UNFINISHABLE step
// (observed for reducible operators inside subterm goals) reports that lemma as
//   `<name> (<kind>): analysis cannot be finished (reducible operators in subterms) (<N> steps)`
// This is a fourth non-diff verdict phrase distinct from verified / falsified /
// analysis incomplete. It renders uniformly for both trace-kinds (no falsified
// suffix) and carries NO accompanying warning/advisory line. Captured from the
// GPL example csf23-subterms/YellowTest.spthy (used only as an observation input).

#[test]
fn summary_reducible_operators_whole_theory_both_kinds() {
    // YellowTest --prove: two lemmas close normally (verified exists-trace,
    // falsified all-traces) and two hit the reducible-operators wall — one
    // exists-trace, one all-traces — both printing the identical uniform phrase.
    // No warning section, so no advisory line accompanies the unfinishable state.
    let out_cap = include_str!("fixtures/r7_yellow_prove.out.txt");
    let err_cap = include_str!("fixtures/r7_yellow_prove.err.txt");
    let streams = frame_single(
        out_cap,
        err_cap,
        "YellowTest",
        None,
        vec![
            lemma("GreenYellow", TraceKind::ExistsTrace, LemmaResult::Verified, 3),
            lemma("RedYellow", TraceKind::AllTraces, LemmaResult::Falsified, 3),
            lemma("YellowRed", TraceKind::ExistsTrace, LemmaResult::AnalysisCannotBeFinished, 4),
            lemma("YellowGreen", TraceKind::AllTraces, LemmaResult::AnalysisCannotBeFinished, 4),
        ],
    );
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
}

#[test]
fn summary_bound_exhaustion_is_incomplete_not_unfinishable() {
    // The SAME theory under `--prove --bound=2`: the bound cuts every proof short
    // BEFORE it can reach the UNFINISHABLE step, so every lemma reads
    // `analysis incomplete (<N> steps)` — never the reducible-operators phrase.
    // Pins that a bounded cutoff and a reducible-operators wall are distinct
    // verdicts even for the identical lemmas.
    let out_cap = include_str!("fixtures/r7_yellow_bound.out.txt");
    let err_cap = include_str!("fixtures/r7_yellow_bound.err.txt");
    let streams = frame_single(
        out_cap,
        err_cap,
        "YellowTest",
        None,
        vec![
            lemma("GreenYellow", TraceKind::ExistsTrace, LemmaResult::AnalysisIncomplete, 4),
            lemma("RedYellow", TraceKind::AllTraces, LemmaResult::AnalysisIncomplete, 4),
            lemma("YellowRed", TraceKind::ExistsTrace, LemmaResult::AnalysisIncomplete, 4),
            lemma("YellowGreen", TraceKind::AllTraces, LemmaResult::AnalysisIncomplete, 4),
        ],
    );
    assert_eq!(streams.out, out_cap);
    assert_eq!(streams.err, err_cap);
    // And the reducible-operators phrase must NOT appear anywhere in the bounded run.
    assert!(!streams.out.contains("cannot be finished"), "bounded run must be plain incomplete");
}

#[test]
fn diff_summary_projected_reducible_operators_both_sides_and_kinds() {
    // YellowDiffTest --diff --prove: the reducible-operators verdict composes with
    // the projected `LHS`/`RHS` prefixes and both trace-kinds. Here every LHS
    // projection hits the wall while the RHS projections close (verified/falsified),
    // proving the phrase is rendered by the same shared verdict path as a
    // whole-theory lemma, with the DiffLemma independently falsified.
    let out_cap = include_str!("fixtures/r7_yellowdiff_prove.out.txt");
    assert_diff_summary(
        out_cap,
        None,
        vec![
            rhs("GreenYellow", TraceKind::ExistsTrace, LemmaResult::Verified, 3),
            lhs("GreenYellow", TraceKind::ExistsTrace, LemmaResult::AnalysisCannotBeFinished, 3),
            rhs("RedYellow", TraceKind::AllTraces, LemmaResult::Falsified, 3),
            lhs("RedYellow", TraceKind::AllTraces, LemmaResult::AnalysisCannotBeFinished, 3),
            rhs("YellowRed", TraceKind::ExistsTrace, LemmaResult::Falsified, 3),
            lhs("YellowRed", TraceKind::ExistsTrace, LemmaResult::AnalysisCannotBeFinished, 3),
            rhs("YellowGreen", TraceKind::AllTraces, LemmaResult::Verified, 3),
            lhs("YellowGreen", TraceKind::AllTraces, LemmaResult::AnalysisCannotBeFinished, 3),
            diff_lemma("Observational_equivalence", LemmaResult::Falsified, 9),
        ],
    );
}

#[test]
fn reducible_operators_line_bytes_are_exact() {
    // Pin the exact reducible-operators phrase independent of any capture: it is
    // byte-identical for exists-trace and all-traces (no falsified-style suffix),
    // and composes unchanged behind a `LHS :  ` projected prefix.
    let summary = Summary {
        analyzed: s("U.spthy"),
        output: None,
        processing_time: 0.04,
        warnings: None,
        lemmas: vec![
            lemma("a", TraceKind::ExistsTrace, LemmaResult::AnalysisCannotBeFinished, 4),
            lemma("b", TraceKind::AllTraces, LemmaResult::AnalysisCannotBeFinished, 4),
            lhs("c", TraceKind::AllTraces, LemmaResult::AnalysisCannotBeFinished, 3),
        ],
    };
    let block = render_summary(std::slice::from_ref(&summary));
    let expected_body = concat!(
        "  processing time: 0.04s\n",
        "  \n",
        "  a (exists-trace): analysis cannot be finished (reducible operators in subterms) (4 steps)\n",
        "  b (all-traces): analysis cannot be finished (reducible operators in subterms) (4 steps)\n",
        "  LHS :  c (all-traces): analysis cannot be finished (reducible operators in subterms) (3 steps)\n",
    );
    assert!(block.contains(expected_body), "block was: {block:?}");
}

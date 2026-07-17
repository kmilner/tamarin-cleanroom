//! Byte-parity tests against captured oracle output.
//!
//! Fixtures in `tests/fixtures/` are raw captures of the black-box binary. The
//! `split_*` fixtures were captured with stdout and stderr redirected separately
//! (`1>out 2>err`); the framing tests reassemble BOTH streams and compare them
//! byte-for-byte. The `vv_*` fixtures are the value-validation error texts.

use cli_clean::args::{parse_args, read_haskell_int, Args, OutputModule, Parsed, PartialEval, StopOnTrace};
use cli_clean::errors::{app_error, open_file_error, OpenFileError, ParseError};
use cli_clean::framing::{
    frame_batch, frame_parse_only, frame_variants, render_summary, BatchTheory, LemmaResult,
    LoadedTheory, Summary, SummaryEntry, TraceKind,
};
use cli_clean::stream::Stream;
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

fn lemma(name: &str, kind: TraceKind, result: LemmaResult, steps: u64) -> SummaryEntry {
    SummaryEntry::Lemma { name: s(name), kind, result, steps }
}

fn nslpk3_entries(nonce_result: LemmaResult, nonce_steps: u64) -> Vec<SummaryEntry> {
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
            entries: nslpk3_entries(LemmaResult::AnalysisIncomplete, 1),
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
            entries: nslpk3_entries(LemmaResult::Verified, 54),
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
            entries: nslpk3_entries(LemmaResult::AnalysisIncomplete, 1),
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
            entries: nslpk3_entries(LemmaResult::AnalysisIncomplete, 1),
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
            entries: vec![SummaryEntry::Warning { count: 1 }],
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
        entries: vec![],
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

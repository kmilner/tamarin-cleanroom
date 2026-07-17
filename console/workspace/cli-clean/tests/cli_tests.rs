//! Byte-parity tests against captured oracle output.
//!
//! Fixtures in `tests/fixtures/` are raw captures of the black-box binary. The
//! framing and error-text tests compare assembled output against INDEPENDENT
//! full-run captures (not the source the renderers were built from), so they are
//! genuine byte-parity checks of the assembly logic.

use cli_clean::errors::{app_error, open_file_error, OpenFileError};
use cli_clean::framing::{
    frame_parse_only, frame_prefix, render_summary, LemmaResult, SummaryEntry, TraceKind,
};
use cli_clean::version::{MaudeInfo, VersionInfo};
use cli_clean::{parse, render_help, render_version, Command, Mode};

fn s(a: &str) -> String {
    a.to_string()
}
fn argv(items: &[&str]) -> Vec<String> {
    items.iter().map(|x| x.to_string()).collect()
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
fn version_banner_is_byte_exact() {
    assert_eq!(render_version(&observed_version()), include_str!("fixtures/version.txt"));
    // -V produces the same output.
    assert_eq!(render_version(&observed_version()), include_str!("fixtures/short_version.txt"));
}

// ---- parse errors -------------------------------------------------------------

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
    // `-h` is unknown; help is `-?`/`--help`.
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
    // A valid flag but no file -> "no input files given" + full global help.
    let e = parse(&argv(&["--stop-on-trace=XYZ"])).unwrap_err();
    assert_eq!(e.text, include_str!("fixtures/err_stopontrace_bad.txt"));
    // --diff with no file gives the same envelope.
    let e2 = parse(&argv(&["--diff"])).unwrap_err();
    assert_eq!(e2.text, include_str!("fixtures/err_stopontrace_bad.txt"));
}

#[test]
fn version_flag_unknown_outside_default_mode() {
    let e = parse(&argv(&["variants", "--version"])).unwrap_err();
    assert_eq!(e.text, "Unknown flag: --version\n");
}

// ---- mode selection & flag parsing semantics ---------------------------------

#[test]
fn help_selects_active_mode() {
    assert_eq!(parse(&argv(&["--help"])).unwrap(), Command::Help(Mode::Batch));
    assert_eq!(parse(&argv(&["interactive", "--help"])).unwrap(), Command::Help(Mode::Interactive));
    assert_eq!(parse(&argv(&["int", "--help"])).unwrap(), Command::Help(Mode::Interactive));
    assert_eq!(parse(&argv(&["v", "--help"])).unwrap(), Command::Help(Mode::Variants));
    assert_eq!(parse(&argv(&["t", "--help"])).unwrap(), Command::Help(Mode::Test));
}

#[test]
fn version_flag_in_default_mode() {
    assert_eq!(parse(&argv(&["--version"])).unwrap(), Command::Version);
    assert_eq!(parse(&argv(&["-V"])).unwrap(), Command::Version);
}

#[test]
fn flag_before_token_keeps_default_mode() {
    // `-v interactive` -> default batch mode; "interactive" is a FILE positional.
    let cmd = parse(&argv(&["-v", "interactive"])).unwrap();
    match cmd {
        Command::Run(spec) => {
            assert_eq!(spec.mode, Mode::Batch);
            assert_eq!(spec.positional, vec![s("interactive")]);
            assert!(spec.options.is_set("verbose"));
        }
        other => panic!("expected Run, got {other:?}"),
    }
}

#[test]
fn non_mode_token_is_a_file() {
    let cmd = parse(&argv(&["foo"])).unwrap();
    match cmd {
        Command::Run(spec) => {
            assert_eq!(spec.mode, Mode::Batch);
            assert_eq!(spec.positional, vec![s("foo")]);
        }
        other => panic!("expected Run, got {other:?}"),
    }
}

#[test]
fn short_attached_value_is_captured() {
    // `-babc` -> bound=abc (attached short value).
    let cmd = parse(&argv(&["file.spthy", "-babc"])).unwrap();
    match cmd {
        Command::Run(spec) => assert_eq!(spec.options.values("bound"), vec!["abc"]),
        other => panic!("expected Run, got {other:?}"),
    }
}

#[test]
fn following_token_is_not_consumed_as_value() {
    // `--bound 5` does NOT consume "5"; it becomes a second positional file.
    let cmd = parse(&argv(&["file.spthy", "--bound", "5"])).unwrap();
    match cmd {
        Command::Run(spec) => {
            assert!(spec.options.is_set("bound"));
            assert_eq!(spec.options.values("bound"), Vec::<&str>::new()); // bare
            assert_eq!(spec.positional, vec![s("file.spthy"), s("5")]);
        }
        other => panic!("expected Run, got {other:?}"),
    }
}

#[test]
fn long_alias_is_recognized() {
    let cmd = parse(&argv(&["file.spthy", "--oj=/tmp/x.json"])).unwrap();
    match cmd {
        Command::Run(spec) => assert_eq!(spec.options.values("output-json"), vec!["/tmp/x.json"]),
        other => panic!("expected Run, got {other:?}"),
    }
}

#[test]
fn multiple_files_accepted() {
    let cmd = parse(&argv(&["a.spthy", "b.spthy"])).unwrap();
    match cmd {
        Command::Run(spec) => assert_eq!(spec.positional, vec![s("a.spthy"), s("b.spthy")]),
        other => panic!("expected Run, got {other:?}"),
    }
}

// ---- batch framing ------------------------------------------------------------

const NSLPK3_PATH: &str = "/home/kamilner/tamarin-cleanroom/console/oracle/examples/classic/NSLPK3.spthy";

#[test]
fn batch_framing_prefix_and_suffix_match_full_run() {
    let fixture = include_str!("fixtures/batch_nslpk3_default.txt");
    let maude = MaudeInfo::default();

    // Prefix: maude preamble + the five progress lines.
    let prefix = frame_prefix(&maude, "NSLPK3");
    assert!(
        fixture.starts_with(&prefix),
        "framing prefix (preamble+progress) must match the real run"
    );

    // Suffix: blank line + summary block.
    let entries = vec![
        SummaryEntry::Lemma {
            name: s("types"),
            kind: TraceKind::AllTraces,
            result: LemmaResult::AnalysisIncomplete,
            steps: 1,
        },
        SummaryEntry::Lemma {
            name: s("nonce_secrecy"),
            kind: TraceKind::AllTraces,
            result: LemmaResult::AnalysisIncomplete,
            steps: 1,
        },
        SummaryEntry::Lemma {
            name: s("injective_agree"),
            kind: TraceKind::AllTraces,
            result: LemmaResult::AnalysisIncomplete,
            steps: 1,
        },
        SummaryEntry::Lemma {
            name: s("session_key_setup_possible"),
            kind: TraceKind::ExistsTrace,
            result: LemmaResult::AnalysisIncomplete,
            steps: 1,
        },
    ];
    let suffix = format!("\n{}", render_summary(NSLPK3_PATH, 0.39, &entries));
    assert!(fixture.ends_with(&suffix), "summary block must match the real run byte-for-byte");
}

#[test]
fn parse_only_framing_has_no_preamble_or_summary() {
    let fixture = include_str!("fixtures/batch_nslpk3_parseonly.txt");
    assert!(fixture.starts_with("[Theory NSLPK3] Theory loaded\n"));
    assert!(!fixture.contains("maude tool"));
    assert!(!fixture.contains("summary of summaries"));
    // Re-framing the opaque payload reproduces the capture.
    let payload = fixture.strip_prefix("[Theory NSLPK3] Theory loaded\n").unwrap();
    assert_eq!(frame_parse_only("NSLPK3", payload), fixture);
}

#[test]
fn warning_summary_line_matches() {
    let fixture = include_str!("fixtures/round2_multiplication_in_rule_lhs.txt");
    let path = "/home/kamilner/tamarin-cleanroom/console/oracle/examples/round2/multiplication_in_rule_lhs.spthy";
    let entries = vec![SummaryEntry::Warning { count: 2 }];
    let suffix = format!("\n{}", render_summary(path, 0.12, &entries));
    assert!(fixture.ends_with(&suffix), "warning summary must match the real run");
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
    assert_eq!(
        open_file_error(dir_path, OpenFileError::IsDirectory).trim_end(),
        last_line(isdir)
    );
}

#[test]
fn app_error_callstack_matches() {
    // The block after the maude preamble (last 3 lines) must match.
    let bound = include_str!("fixtures/err_bound_nonint.txt");
    assert!(bound.ends_with(&app_error("bound: invalid bound given")));
    let outmod = include_str!("fixtures/err_output_module_bad.txt");
    assert!(outmod.ends_with(&app_error("output mode not supported.")));
}

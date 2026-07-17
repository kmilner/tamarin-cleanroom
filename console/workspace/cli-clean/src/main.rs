//! Thin driver exercising the CLI text surface.
//!
//! Handles the pure-text paths (help, version, parse errors) byte-for-byte so
//! they can be verified against the black-box binary. `Run` has no prover engine
//! behind it here — real integration wires the parsed [`cli_clean::RunSpec`] into
//! the prover and uses [`cli_clean::framing`] to wrap its output.

use std::process::exit;

use cli_clean::version::{MaudeInfo, VersionInfo};
use cli_clean::{parse, render_version, Command};

/// Version slots as observed from the reference build. Real integration injects
/// the actual build metadata of the hosting binary.
fn observed_version_info() -> VersionInfo {
    VersionInfo {
        maude: MaudeInfo::default(),
        tamarin_version: "1.13.0".to_string(),
        copyright_years: "2010-2023".to_string(),
        git_description: "0234f6a1abee25677c0accef62de0ac2883b0347 (with uncommited changes), branch: HEAD"
            .to_string(),
        compiled_at: "2026-07-16 18:33:16.036427196 UTC".to_string(),
    }
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    match parse(&argv) {
        Ok(Command::Help(mode)) => {
            print!("{}", cli_clean::render_help(mode));
            exit(0);
        }
        Ok(Command::Version) => {
            print!("{}", render_version(&observed_version_info()));
            exit(0);
        }
        Ok(Command::Run(spec)) => {
            // Text-surface stub: no prover engine in this crate.
            eprintln!(
                "[cli-clean] parsed: mode={:?} positional={:?} flags={:?}",
                spec.mode, spec.positional, spec.options.flags
            );
            exit(0);
        }
        Err(e) => {
            eprint!("{}", e.text);
            exit(e.exit_code);
        }
    }
}

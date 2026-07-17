//! Thin driver exercising the CLI text surface.
//!
//! Handles the pure-text paths (help, version, parse/validation errors)
//! byte-for-byte so they can be verified against the black-box binary. `Run` has
//! no prover engine behind it here — real integration wires the typed
//! [`cli_clean::Args`] into the prover and uses [`cli_clean::framing`] to wrap its
//! output across the two streams.

use std::process::exit;

use cli_clean::stream::Stream;
use cli_clean::version::{frame_version, MaudeInfo, VersionInfo};
use cli_clean::{parse_args, Parsed};

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
    match parse_args(&argv) {
        Ok(Parsed::Help(mode)) => {
            print!("{}", cli_clean::render_help(mode));
            exit(0);
        }
        Ok(Parsed::Version) => {
            let streams = frame_version(&observed_version_info());
            print!("{}", streams.out);
            eprint!("{}", streams.err);
            exit(0);
        }
        Ok(Parsed::Run(args)) => {
            // Text-surface stub: no prover engine in this crate.
            eprintln!(
                "[cli-clean] parsed: mode={:?} positional={:?} bound={:?} stop_on_trace={:?} output_module={:?} processors={:?}",
                args.mode,
                args.positional,
                args.bound,
                args.stop_on_trace,
                args.output_module,
                args.processors,
            );
            exit(0);
        }
        Err(e) => {
            match e.stream {
                Stream::Out => print!("{}", e.text),
                Stream::Err => eprint!("{}", e.text),
            }
            exit(e.exit_code);
        }
    }
}

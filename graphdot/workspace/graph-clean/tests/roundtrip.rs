//! Byte-parity round-trip tests against captured DOT payloads.
//!
//! The shared parser (`parse_util`) reads each captured fixture into the crate's
//! own [`Graph`] model; re-serializing with [`graph_clean::dot::to_dot`] must
//! reproduce the original bytes exactly. This validates the payload-format spec
//! (BEHAVIOR.md §2, §3) end to end.

use graph_clean::dot::to_dot;

#[path = "parse_util.rs"]
mod parse_util;
use parse_util::parse;

fn check(name: &str, dot: &str) {
    let g = parse(dot);
    let out = to_dot(&g);
    assert_eq!(out, dot, "byte round-trip mismatch for fixture {name}");
    // The inferred header must match what the fixture actually uses.
    assert_eq!(g.infer_header(), g.header, "header trigger mismatch for {name}");
}

#[test]
fn roundtrip_empty() {
    check("empty", include_str!("fixtures/empty.dot"));
}

#[test]
fn roundtrip_simple_ports() {
    check("simple_ports", include_str!("fixtures/simple_ports.dot"));
}

#[test]
fn roundtrip_simple_abbrev() {
    check("simple_abbrev", include_str!("fixtures/simple_abbrev.dot"));
}

#[test]
fn roundtrip_compact_clusters() {
    check("compact_clusters", include_str!("fixtures/compact_clusters.dot"));
}

#[test]
fn roundtrip_compact_abbrev() {
    check("compact_abbrev", include_str!("fixtures/compact_abbrev.dot"));
}

#[test]
fn roundtrip_multi_abbrev() {
    check("multi_abbrev", include_str!("fixtures/multi_abbrev.dot"));
}

#[test]
fn roundtrip_nsl_invtrap() {
    // A live-probed graph carrying an invtrapezium `(#i, 0)` open-target node.
    check("nsl_invtrap", include_str!("fixtures/nsl_invtrap.dot"));
}

#[test]
fn roundtrip_invtrap_compressed() {
    // Compressed NAXOS case graph: invtrapezium + gray !KU ellipse + wrapped cell.
    check("invtrap_compressed", include_str!("fixtures/invtrap_compressed.dot"));
}

#[test]
fn roundtrip_invtrap_raw() {
    // Fully uncompressed + unabbreviated variant (simplification level 0).
    check("invtrap_raw", include_str!("fixtures/invtrap_raw.dot"));
}

#[test]
fn roundtrip_cluster_process() {
    // Live SAPIC single-cluster graph.
    check("cluster_process", include_str!("fixtures/cluster_process.dot"));
}

#[test]
fn roundtrip_cluster_multi() {
    // Free ellipses + one role cluster + deduction edges.
    check("cluster_multi", include_str!("fixtures/cluster_multi.dot"));
}

#[test]
fn roundtrip_last_timepoint() {
    // The `#last` designated-timepoint ellipse and its before-edge.
    check("last_timepoint", include_str!("fixtures/last_timepoint.dot"));
}

#[test]
fn roundtrip_wrap_e12() {
    // A wrapped record cell (literal `\l` / `&nbsp;`) round-trips byte-exact.
    check("wrap_e12", include_str!("fixtures/wrap_E12.dot"));
}

/// Bulk generalization check (off by default). Set `GRAPHCLEAN_CORPUS` to a
/// directory of captured `*.dot` payloads; every one must round-trip byte-exact.
#[test]
fn roundtrip_corpus_dir() {
    let Ok(dir) = std::env::var("GRAPHCLEAN_CORPUS") else {
        return;
    };
    let mut total = 0usize;
    let mut ok = 0usize;
    let mut fails: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("dot") {
            continue;
        }
        let dot = std::fs::read_to_string(&path).unwrap();
        total += 1;
        let out = std::panic::catch_unwind(|| to_dot(&parse(&dot)))
            .unwrap_or_else(|_| String::from("<PANIC>"));
        if out == dot {
            ok += 1;
        } else if fails.len() < 20 {
            fails.push(path.file_name().unwrap().to_string_lossy().into_owned());
        }
    }
    eprintln!("corpus round-trip: {ok}/{total} byte-exact");
    for f in &fails {
        eprintln!("  MISMATCH: {f}");
    }
    assert_eq!(ok, total, "{} of {} payloads failed to round-trip", total - ok, total);
}

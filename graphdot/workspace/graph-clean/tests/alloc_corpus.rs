//! Corpus validation of the node-id/port ALLOCATOR (BEHAVIOR.md §3e, item 3).
//!
//! Re-uses the round-trip parser to read a captured graph into the model (which
//! preserves the original `n<K>` ids and ports), then re-derives every id from
//! scratch with [`NodeIdAllocator`] by walking the statements in emission order.
//! If the allocator's scheme is right, the re-derived ids equal the captured
//! ones for every node and every port.
//!
//! Off by default; set `GRAPHCLEAN_CORPUS` to a directory of `*.dot` payloads.
//! The six committed fixtures are always checked by `alloc_fixtures`.

use graph_clean::alloc::NodeIdAllocator;
use graph_clean::model::*;

#[path = "parse_util.rs"]
mod parse_util;
use parse_util::parse;

/// Walk `stmts` in emission order, allocating ids, and assert they match the ids
/// the parser preserved from the payload. Recurses into clusters / rank blocks at
/// their position (the global counter does not reset).
fn check_alloc(stmts: &[Stmt], alloc: &mut NodeIdAllocator) -> Result<(), String> {
    for s in stmts {
        match s {
            Stmt::Node(n) => match &n.kind {
                NodeKind::Record(r) => {
                    let n_cells: usize = r.columns.iter().map(|c| c.len()).sum();
                    let ids = alloc.record(n_cells);
                    // ports, in cell order across groups
                    let mut got_ports = ids.ports.into_iter();
                    for col in &r.columns {
                        for cell in col {
                            let want = got_ports.next().unwrap();
                            if cell.port != want {
                                return Err(format!("port {} != {}", cell.port, want));
                            }
                        }
                    }
                    if n.id != ids.node {
                        return Err(format!("record node {} != {}", n.id, ids.node));
                    }
                }
                _ => {
                    let id = alloc.node();
                    if n.id != id {
                        return Err(format!("node {} != {}", n.id, id));
                    }
                }
            },
            Stmt::Cluster(c) => check_alloc(&c.body, alloc)?,
            Stmt::RankBlock(b) => check_alloc(&b.body, alloc)?,
            Stmt::Edge(_) => {}
        }
    }
    Ok(())
}

fn check_payload(dot: &str) -> Result<(), String> {
    let g = parse(dot);
    let mut alloc = NodeIdAllocator::new();
    check_alloc(&g.body, &mut alloc)
}

#[test]
fn alloc_fixtures() {
    for (name, dot) in [
        ("simple_ports", include_str!("fixtures/simple_ports.dot") as &str),
        ("simple_abbrev", include_str!("fixtures/simple_abbrev.dot")),
        ("compact_clusters", include_str!("fixtures/compact_clusters.dot")),
        ("compact_abbrev", include_str!("fixtures/compact_abbrev.dot")),
        ("multi_abbrev", include_str!("fixtures/multi_abbrev.dot")),
        ("nsl_invtrap", include_str!("fixtures/nsl_invtrap.dot")),
        ("invtrap_compressed", include_str!("fixtures/invtrap_compressed.dot")),
    ] {
        check_payload(dot).unwrap_or_else(|e| panic!("alloc mismatch in {name}: {e}"));
    }
}

#[test]
fn alloc_corpus_dir() {
    let Ok(dir) = std::env::var("GRAPHCLEAN_CORPUS") else {
        return;
    };
    let (mut total, mut ok) = (0usize, 0usize);
    let mut fails: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("dot") {
            continue;
        }
        let dot = std::fs::read_to_string(&path).unwrap();
        total += 1;
        match std::panic::catch_unwind(|| check_payload(&dot)) {
            Ok(Ok(())) => ok += 1,
            other if fails.len() < 20 => {
                fails.push(format!("{}: {:?}", path.file_name().unwrap().to_string_lossy(), other));
            }
            _ => {}
        }
    }
    eprintln!("allocator corpus check: {ok}/{total} byte-consistent");
    for f in &fails {
        eprintln!("  MISMATCH {f}");
    }
    assert_eq!(ok, total, "{} of {} graphs violated the allocation scheme", total - ok, total);
}

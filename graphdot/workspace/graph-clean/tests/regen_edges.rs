//! Round-13 REGENERATION tests for the edge layer (BEHAVIOR.md §3c, §6).
//!
//! Unlike the opaque round-trip (which preserves parsed edge attributes and
//! port strings verbatim), these tests rebuild each graph's edge set from a
//! STRUCTURED [`System`] — every edge is a typed [`SysEdge`] carrying an
//! [`EndRef`] endpoint reference and an [`EdgeStyle`] — and assert that
//! [`generate`] re-emits the payload's edge section byte-for-byte. This
//! exercises the full (color, style) vocabulary and the record info-port anchor
//! ([`EndRef::Info`]) through the real id/port allocator.

use graph_clean::dot::to_dot;
use graph_clean::generate::*;
use graph_clean::model::*;
use std::collections::{BTreeSet, HashMap};

#[path = "parse_util.rs"]
mod parse_util;
use parse_util::parse;

/// The complete observed edge-style vocabulary and the exact attribute bracket
/// each renders (§3c census over the whole 12 022-file corpus: these eleven
/// (color, style) attribute sets are the entire observed set, with counts
/// red/dashed 147937, bold 86531, bold+gray50 86298, gray30 50359,
/// blue3/dashed 37726, orangered2 29917, invis 25271, black/dashed 6909,
/// purple/dashed 2416, dotted/green 2394, darkorange3/dashed 1553).
const STYLE_VOCAB: &[(EdgeStyle, &str)] = &[
    (EdgeStyle::StructuralGray, "[style=\"bold\",weight=\"10.0\",color=\"gray50\"]"),
    (EdgeStyle::Structural, "[style=\"bold\",weight=\"10.0\"]"),
    (EdgeStyle::Message, "[color=\"gray30\"]"),
    (EdgeStyle::KnowledgeDeduction, "[color=\"red\",style=\"dashed\"]"),
    (EdgeStyle::Deduction, "[color=\"orangered2\"]"),
    (EdgeStyle::TemporalBlue, "[color=\"blue3\",style=\"dashed\"]"),
    (EdgeStyle::TemporalBlack, "[color=\"black\",style=\"dashed\"]"),
    // The three round-13 additions:
    (EdgeStyle::PurpleDashed, "[color=\"purple\",style=\"dashed\"]"),
    (EdgeStyle::GreenDotted, "[style=\"dotted\",color=\"green\"]"),
    (EdgeStyle::DarkorangeDashed, "[color=\"darkorange3\",style=\"dashed\"]"),
    (EdgeStyle::Invis, "[style=\"invis\"]"),
];

/// Every style in the vocabulary renders to its exact captured attribute bracket
/// (note `dotted`/`green` is the sole `style`-before-`color` ordering).
#[test]
fn edge_style_vocabulary_renders_byte_exact() {
    for (style, bracket) in STYLE_VOCAB {
        let sys = System {
            nodes: vec![
                GraphNode::Temporal { var: "a".into() },
                GraphNode::Temporal { var: "b".into() },
            ],
            edges: vec![SysEdge::new(EndRef::Whole(0), EndRef::Whole(1), *style)],
            ..System::default()
        };
        let dot = to_dot(&generate(&sys));
        let want = format!("n0 -> n1{};", bracket);
        assert!(dot.contains(&want), "style {:?} must render `{}`; got:\n{}", style, want, dot);
    }
}

/// The vocabulary is exactly the observed set: every distinct edge bracket in
/// the committed regeneration corpus is one of the eleven, and all eleven occur.
#[test]
fn fixture_corpus_uses_only_the_known_vocabulary() {
    let known: BTreeSet<&str> = STYLE_VOCAB.iter().map(|(_, b)| *b).collect();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for (_, dot) in regen_fixtures() {
        for line in dot.lines().filter(|l| l.contains(" -> ")) {
            if let (Some(o), Some(c)) = (line.find('['), line.rfind(']')) {
                seen.insert(line[o..=c].to_string());
            }
        }
    }
    for b in &seen {
        assert!(known.contains(b.as_str()), "capture uses an un-modeled edge bracket: {}", b);
    }
    for b in &known {
        assert!(seen.contains(*b), "fixture corpus is missing observed style {}", b);
    }
}

// ---------------------------------------------------------------------------
// Regeneration: rebuild each capture's edge section from a structured System.
// ---------------------------------------------------------------------------

/// A parsed node's id footprint, enough to reproduce its id/port allocation.
enum Foot {
    Record { nprem: usize, nconcl: usize },
    Single,
}

/// The role of a record port, i.e. which [`EndRef`] addresses it.
#[derive(Clone, Copy)]
enum PortRole {
    Prem(usize),
    Info,
    Concl(usize),
}

/// One parsed node with its integer id and, for records, its per-role ports.
struct NodeInfo {
    node_id: usize,
    foot: Foot,
    prem_ports: Vec<usize>,
    info_port: Option<usize>,
    concl_ports: Vec<usize>,
}

fn nid(s: &str) -> usize {
    s.trim_start_matches('n').parse().unwrap_or_else(|_| panic!("bad id {s}"))
}

fn node_info(n: &Node) -> NodeInfo {
    let node_id = nid(&n.id);
    match &n.kind {
        NodeKind::Record(r) => {
            // The info cell is the record's single middle-group cell; its text is
            // the only one that begins with a temporal `#` sigil (facts never do).
            let info_col = r
                .columns
                .iter()
                .position(|col| col.first().is_some_and(|c| c.text.starts_with('#')))
                .expect("record must have an info cell");
            let mut prem_ports = Vec::new();
            let mut concl_ports = Vec::new();
            let mut info_port = None;
            for (ci, col) in r.columns.iter().enumerate() {
                if ci < info_col {
                    prem_ports.extend(col.iter().map(|c| nid(&c.port)));
                } else if ci == info_col {
                    info_port = Some(nid(&col[0].port));
                } else {
                    concl_ports.extend(col.iter().map(|c| nid(&c.port)));
                }
            }
            let foot = Foot::Record { nprem: prem_ports.len(), nconcl: concl_ports.len() };
            NodeInfo { node_id, foot, prem_ports, info_port, concl_ports }
        }
        // Ellipse / shaped / plain (legend) each occupy a single id.
        _ => NodeInfo { node_id, foot: Foot::Single, prem_ports: Vec::new(), info_port: None, concl_ports: Vec::new() },
    }
}

fn collect(stmts: &[Stmt], nodes: &mut Vec<NodeInfo>, edges: &mut Vec<Edge>) {
    for s in stmts {
        match s {
            Stmt::Node(n) => nodes.push(node_info(n)),
            Stmt::Edge(e) => edges.push(e.clone()),
            Stmt::Cluster(c) => collect(&c.body, nodes, edges),
            Stmt::RankBlock(b) => collect(&b.body, nodes, edges),
        }
    }
}

fn map_style(attrs: &[(String, String)]) -> EdgeStyle {
    let a: Vec<(&str, &str)> = attrs.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    match a.as_slice() {
        [("style", "bold"), ("weight", "10.0"), ("color", "gray50")] => EdgeStyle::StructuralGray,
        [("style", "bold"), ("weight", "10.0")] => EdgeStyle::Structural,
        [("color", "gray30")] => EdgeStyle::Message,
        [("color", "red"), ("style", "dashed")] => EdgeStyle::KnowledgeDeduction,
        [("color", "orangered2")] => EdgeStyle::Deduction,
        [("color", "blue3"), ("style", "dashed")] => EdgeStyle::TemporalBlue,
        [("color", "black"), ("style", "dashed")] => EdgeStyle::TemporalBlack,
        [("color", "purple"), ("style", "dashed")] => EdgeStyle::PurpleDashed,
        [("style", "dotted"), ("color", "green")] => EdgeStyle::GreenDotted,
        [("color", "darkorange3"), ("style", "dashed")] => EdgeStyle::DarkorangeDashed,
        [("style", "invis")] => EdgeStyle::Invis,
        _ => panic!("un-modeled edge style {attrs:?}"),
    }
}

/// Result of rebuilding one capture: the reconstructed and captured edge-line
/// sections, plus coverage flags for the acceptance assertions.
struct Rebuilt {
    got: Vec<String>,
    want: Vec<String>,
    used_info: bool,
    styles: BTreeSet<String>,
}

fn rebuild(dot: &str) -> Rebuilt {
    let g = parse(dot);
    let mut nodes: Vec<NodeInfo> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    collect(&g.body, &mut nodes, &mut edges);

    // Allocation order == ascending node id (the allocator hands out contiguous
    // id blocks in emission order, so node ids increase monotonically).
    nodes.sort_by_key(|ni| ni.node_id);
    let mut id_to_index: HashMap<usize, usize> = HashMap::new();
    let mut port_to_ref: HashMap<usize, (usize, PortRole)> = HashMap::new();
    for (idx, ni) in nodes.iter().enumerate() {
        id_to_index.insert(ni.node_id, idx);
        for (i, p) in ni.prem_ports.iter().enumerate() {
            port_to_ref.insert(*p, (idx, PortRole::Prem(i)));
        }
        if let Some(ip) = ni.info_port {
            port_to_ref.insert(ip, (idx, PortRole::Info));
        }
        for (i, p) in ni.concl_ports.iter().enumerate() {
            port_to_ref.insert(*p, (idx, PortRole::Concl(i)));
        }
    }

    // Placeholder nodes reproducing only the id/port footprint (cell text is
    // irrelevant to the edge section, which is all this test compares).
    let sys_nodes: Vec<GraphNode> = nodes
        .iter()
        .map(|ni| match ni.foot {
            Foot::Record { nprem, nconcl } => GraphNode::RawRule(
                RawRule::new("#t : r", "#000000")
                    .premises(vec!["p".to_string(); nprem])
                    .conclusions(vec!["c".to_string(); nconcl]),
            ),
            Foot::Single => GraphNode::Temporal { var: "x".into() },
        })
        .collect();

    let map_end = |ep: &EndPoint| -> EndRef {
        match &ep.port {
            None => EndRef::Whole(id_to_index[&nid(&ep.node)]),
            Some(p) => {
                let (idx, role) = port_to_ref[&nid(p)];
                match role {
                    PortRole::Prem(i) => EndRef::Premise(idx, i),
                    PortRole::Info => EndRef::Info(idx),
                    PortRole::Concl(i) => EndRef::Conclusion(idx, i),
                }
            }
        }
    };

    let mut used_info = false;
    let mut styles = BTreeSet::new();
    let sys_edges: Vec<SysEdge> = edges
        .iter()
        .map(|e| {
            let src = map_end(&e.src);
            let dst = map_end(&e.dst);
            if matches!(src, EndRef::Info(_)) || matches!(dst, EndRef::Info(_)) {
                used_info = true;
            }
            let attrs: Vec<String> =
                e.attrs.iter().map(|(k, v)| format!("{k}=\"{v}\"")).collect();
            styles.insert(format!("[{}]", attrs.join(",")));
            SysEdge::new(src, dst, map_style(&e.attrs))
        })
        .collect();

    let sys = System { nodes: sys_nodes, edges: sys_edges, ..System::default() };
    let out = to_dot(&generate(&sys));

    let got: Vec<String> =
        out.lines().filter(|l| l.contains(" -> ")).map(|s| s.to_string()).collect();
    let want: Vec<String> =
        dot.lines().filter(|l| l.contains(" -> ")).map(|s| s.to_string()).collect();
    Rebuilt { got, want, used_info, styles }
}

/// Load the committed round-13 regeneration fixtures (`tests/fixtures/regen/`).
fn regen_fixtures() -> Vec<(String, String)> {
    let dir = format!("{}/tests/fixtures/regen", env!("CARGO_MANIFEST_DIR"));
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read regen fixtures {dir}: {e}"))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("dot"))
        .collect();
    entries.sort();
    entries
        .into_iter()
        .map(|p| {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            (name, std::fs::read_to_string(&p).unwrap())
        })
        .collect()
}

/// Rebuild the edge section of every committed capture from structured form and
/// assert a byte-exact match, and that the set collectively exercises the record
/// info-port anchor and all three round-13 styles.
#[test]
fn regenerate_edge_sections_byte_exact() {
    let fixtures = regen_fixtures();
    assert!(fixtures.len() >= 20, "need >= 20 diverse captures, have {}", fixtures.len());

    let mut any_info = false;
    let mut styles: BTreeSet<String> = BTreeSet::new();
    for (name, dot) in &fixtures {
        let r = rebuild(dot);
        assert_eq!(r.got, r.want, "edge section mismatch regenerating {name}");
        any_info |= r.used_info;
        styles.extend(r.styles);
    }

    assert!(any_info, "no capture exercised the info-port anchor (EndRef::Info)");
    for bracket in [
        "[color=\"purple\",style=\"dashed\"]",
        "[style=\"dotted\",color=\"green\"]",
        "[color=\"darkorange3\",style=\"dashed\"]",
    ] {
        assert!(styles.contains(bracket), "regeneration set never exercised style {bracket}");
    }
}

/// Bulk generalization (off by default). Set `GRAPHCLEAN_CORPUS` to a directory
/// of captured `*.dot` payloads; every one's edge section must regenerate
/// byte-exact from structured form.
#[test]
fn regenerate_corpus_dir() {
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
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let res = std::panic::catch_unwind(|| {
            let r = rebuild(&dot);
            r.got == r.want
        });
        match res {
            Ok(true) => ok += 1,
            _ if fails.len() < 20 => fails.push(name),
            _ => {}
        }
    }
    eprintln!("edge-section regeneration: {ok}/{total} byte-exact");
    for f in &fails {
        eprintln!("  MISMATCH: {f}");
    }
    assert_eq!(ok, total, "{} of {} payloads failed edge regeneration", total - ok, total);
}

/// Focused witnesses: the two captures the round-13 task pins — the dropped
/// darkorange3/dashed edge and the `n131:n128 -> n4` interior info-port anchor.
#[test]
fn regenerate_named_witnesses() {
    for name in ["01c5db0a7030e664.dot", "00082e1d6a47b5af.dot"] {
        let path = format!("{}/tests/fixtures/regen/{name}", env!("CARGO_MANIFEST_DIR"));
        let dot = std::fs::read_to_string(&path).unwrap();
        let r = rebuild(&dot);
        assert_eq!(r.got, r.want, "witness {name} edge section must regenerate byte-exact");
    }
    // The info-port witness carries the exact interior anchor called out by the task.
    let w = std::fs::read_to_string(format!(
        "{}/tests/fixtures/regen/00082e1d6a47b5af.dot",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let r = rebuild(&w);
    assert!(
        r.got.iter().any(|l| l == "n131:n128 -> n4[color=\"blue3\",style=\"dashed\"];"),
        "info-port anchored edge must regenerate exactly"
    );
    assert!(r.used_info, "the info-port witness must use EndRef::Info");
}

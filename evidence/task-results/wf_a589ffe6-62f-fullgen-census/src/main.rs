//! Open-side FULL-GENERATE census harness (graph round-12).
//!
//! Unlike the round-11 cell-layout-only census (which reused the reference
//! payload's ids/ports/clustering/legend/edges verbatim and re-laid only cell
//! CONTENT), this harness reconstructs a complete clean `generate::System` from
//! each reference payload and drives the ENTIRE clean pipeline:
//!   * `alloc`   — every `n<K>` node/port id is re-derived from scratch;
//!   * clustering — records are re-grouped into `cluster_<label>` subgraphs from
//!                  their role/cluster annotations, in first-appearance order;
//!   * records    — every prem/info/concl cell is re-wrapped (round-12 layout);
//!   * legend     — the `{rank="sink"; …}` block + its invis edges are re-emitted;
//!   * edges      — endpoints resolved through `EndRef`, styles through the fixed
//!                  `EdgeStyle` vocabulary;
//!   * serialize  — `dot::to_dot`.
//! The clean output is then diffed byte-for-byte AND (via a separate Python pass)
//! semantically (web_normalize.canon_dot) against the reference.
//!
//! The ONE component NOT re-derived is the abbreviation SELECTION: the cell text
//! is fed pre-abbreviated (dewrapped from the reference), because selecting which
//! sub-terms to abbreviate needs live `LNTerm`s (the standing `LNTerm ->
//! graph_clean::Term` blocker). Everything else in the pipeline is exercised.
//!
//! Measures INPUTS/measurements only; no clean render logic is reimplemented.

use graph_clean::dot::to_dot;
use graph_clean::generate::{
    generate, ClusterRef, EdgeStyle, EndRef, GraphNode, RawRule, RuleInstance, SysEdge, System,
};
use graph_clean::model::{Graph as MGraph, NodeKind, Stmt};
use std::collections::HashMap;

mod parse_util;
use parse_util::parse;

// ---------------------------------------------------------------------------
// dewrap: undo the reference cell's \l wrapping back to flat text (same law as
// the round-11 census; generate() re-wraps it).
// ---------------------------------------------------------------------------

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == '\\' && i + 1 < cs.len() && matches!(cs[i + 1], '<' | '>' | '{' | '}' | '|') {
            out.push(cs[i + 1]);
            i += 2;
        } else {
            out.push(cs[i]);
            i += 1;
        }
    }
    out
}

fn dewrap(cell: &str) -> String {
    if !cell.contains("\\l") {
        return unescape(cell);
    }
    let raw_lines: Vec<&str> = cell.split("\\l").collect();
    let mut lines: Vec<(bool, String)> = Vec::new();
    for (i, l) in raw_lines.iter().enumerate() {
        if i + 1 == raw_lines.len() && l.is_empty() {
            break;
        }
        let mut rest = *l;
        let mut indented = false;
        while let Some(r) = rest.strip_prefix("&nbsp;") {
            rest = r;
            indented = true;
        }
        lines.push((indented, unescape(rest)));
    }
    if lines.is_empty() {
        return String::new();
    }
    let mut flat = lines[0].1.clone();
    for (indented, li) in &lines[1..] {
        if flat.ends_with(", ") {
            flat.push_str(li);
        } else if flat.ends_with(',') {
            flat.push(' ');
            flat.push_str(li);
        } else if li == ")" && !indented {
            flat.push_str(" )");
        } else {
            flat.push_str(li);
        }
    }
    flat
}

// ---------------------------------------------------------------------------
// Reverse the fixed EdgeStyle vocabulary from an attr list.
// ---------------------------------------------------------------------------

fn attrs_to_style(attrs: &[(String, String)]) -> Option<EdgeStyle> {
    let pairs: Vec<(&str, &str)> = attrs.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let s: &[(&str, &str)] = &pairs;
    let style = match s {
        [("style", "bold"), ("weight", "10.0"), ("color", "gray50")] => EdgeStyle::StructuralGray,
        [("style", "bold"), ("weight", "10.0")] => EdgeStyle::Structural,
        [("color", "gray30")] => EdgeStyle::Message,
        [("color", "red"), ("style", "dashed")] => EdgeStyle::KnowledgeDeduction,
        [("color", "orangered2")] => EdgeStyle::Deduction,
        [("color", "blue3"), ("style", "dashed")] => EdgeStyle::TemporalBlue,
        [("color", "black"), ("style", "dashed")] => EdgeStyle::TemporalBlack,
        [("style", "invis")] => EdgeStyle::Invis,
        _ => return None,
    };
    Some(style)
}

// ---------------------------------------------------------------------------
// Reconstruction: parsed model::Graph -> generate::System.
// Always produces a System (best effort). `compromises` records every place the
// reconstruction could not feed generate() a structurally-faithful input; these
// attribute byte divergences to the adapter rather than to the clean engine.
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Recon {
    sys: System,
    compromises: Vec<&'static str>,
    // reference node id -> EndRef into sys.nodes (whole node)
    whole: HashMap<String, usize>,
    // reference port id -> EndRef (premise/conclusion of a record)
    port: HashMap<String, EndRef>,
    // reference INFO port id -> owning record node idx. generate()'s EndRef has
    // no Info variant, so an edge anchored at an info port is a clean-model gap.
    info_port: HashMap<String, usize>,
    legend_ref_id: Option<String>,
}

/// Split a record's parsed columns into (premise flats, info flat, conclusion
/// flats) by locating the info group (first cell starting with `#`).
fn split_record(
    cols: &[Vec<graph_clean::model::Cell>],
) -> Option<(Vec<(String, String)>, (String, String), Vec<(String, String)>)> {
    let info_group = cols.iter().position(|g| {
        g.first()
            .map(|c| c.text.trim_start().starts_with('#'))
            .unwrap_or(false)
    })?;
    let mut prem = Vec::new();
    for g in &cols[..info_group] {
        for c in g {
            prem.push((c.port.clone(), dewrap(&c.text)));
        }
    }
    let info_cell = &cols[info_group][0];
    let info = (info_cell.port.clone(), dewrap(&info_cell.text));
    let mut concl = Vec::new();
    for g in &cols[info_group + 1..] {
        for c in g {
            concl.push((c.port.clone(), dewrap(&c.text)));
        }
    }
    Some((prem, info, concl))
}

fn add_record(
    r: &mut Recon,
    rec: &graph_clean::model::Record,
    cluster: Option<(&str, &str)>,
) {
    let idx = r.sys.nodes.len();
    let Some((prem, info, concl)) = split_record(&rec.columns) else {
        // No info cell: generate always emits an info group, so this record
        // cannot be reproduced faithfully. Emit an empty-info raw rule so the
        // node still exists for the semantic node-set, and flag it.
        r.compromises.push("record-without-info-group");
        let mut rr = RawRule::new("", &rec.fillcolor);
        rr.fontcolor = rec.fontcolor.clone();
        rr.role = rec.role.0.clone();
        if let Some((l, c)) = cluster {
            rr.cluster = Some(ClusterRef::new(l, c));
        }
        r.sys.nodes.push(GraphNode::RawRule(rr));
        return;
    };
    // Register endpoint map (ports resolve to Premise/Conclusion of this record).
    for (j, (p, _)) in prem.iter().enumerate() {
        r.port.insert(p.clone(), EndRef::Premise(idx, j));
    }
    for (j, (p, _)) in concl.iter().enumerate() {
        r.port.insert(p.clone(), EndRef::Conclusion(idx, j));
    }
    r.info_port.insert(info.0.clone(), idx);
    let mut rr = RawRule::new(&info.1, &rec.fillcolor);
    rr.fontcolor = rec.fontcolor.clone();
    rr.role = rec.role.0.clone();
    rr.premises = prem.into_iter().map(|(_, t)| t).collect();
    rr.conclusions = concl.into_iter().map(|(_, t)| t).collect();
    if let Some((l, c)) = cluster {
        rr.cluster = Some(ClusterRef::new(l, c));
    }
    r.sys.nodes.push(GraphNode::RawRule(rr));
}

fn add_plain_shaped(r: &mut Recon, id: &str, kind: &NodeKind) {
    let idx = r.sys.nodes.len();
    let node = match kind {
        NodeKind::Ellipse(e) => GraphNode::Shaped {
            label: e.label.clone(),
            shape: "ellipse".to_string(),
            color: e.color.clone(),
        },
        NodeKind::Shaped(s) => GraphNode::Shaped {
            label: s.label.clone(),
            shape: s.shape.clone(),
            color: s.color.clone(),
        },
        _ => unreachable!("add_plain_shaped only for ellipse/shaped"),
    };
    r.whole.insert(id.to_string(), idx);
    r.sys.nodes.push(node);
}

/// One pass to collect nodes (building endpoint maps), a second to resolve edges.
fn reconstruct(g: &MGraph) -> Recon {
    let mut r = Recon::default();
    // Pending edges (src, dst, attrs) to resolve after all node maps are built.
    let mut pending: Vec<(graph_clean::model::EndPoint, graph_clean::model::EndPoint, Vec<(String, String)>)> =
        Vec::new();

    // Pass A: nodes.
    for s in &g.body {
        match s {
            Stmt::Node(n) => match &n.kind {
                NodeKind::Record(rec) => {
                    // Free (top-level) record. Register whole-id too (rare edge target).
                    let idx = r.sys.nodes.len();
                    r.whole.insert(n.id.clone(), idx);
                    add_record(&mut r, rec, None);
                }
                NodeKind::Ellipse(_) | NodeKind::Shaped(_) => add_plain_shaped(&mut r, &n.id, &n.kind),
                NodeKind::Plain { .. } => {
                    // A plain (legend) node at top level is not the observed shape
                    // (legend lives in a rank="sink" block); flag it.
                    r.compromises.push("plain-node-at-top-level");
                }
            },
            Stmt::Cluster(c) => {
                for cs in &c.body {
                    match cs {
                        Stmt::Node(n) => match &n.kind {
                            NodeKind::Record(rec) => {
                                let idx = r.sys.nodes.len();
                                r.whole.insert(n.id.clone(), idx);
                                add_record(&mut r, rec, Some((&c.label, &c.color)));
                            }
                            NodeKind::Ellipse(_) | NodeKind::Shaped(_) => {
                                // generate() never clusters a non-record; it goes
                                // free. Include it (semantic node-set) but flag.
                                r.compromises.push("non-record-in-cluster");
                                add_plain_shaped(&mut r, &n.id, &n.kind);
                            }
                            NodeKind::Plain { .. } => r.compromises.push("plain-node-in-cluster"),
                        },
                        Stmt::Edge(_) => r.compromises.push("edge-in-cluster"),
                        _ => r.compromises.push("nested-block-in-cluster"),
                    }
                }
            }
            Stmt::RankBlock(b) => {
                // The legend sink block. Capture the plain node's inner HTML.
                for bs in &b.body {
                    if let Stmt::Node(n) = bs {
                        if let NodeKind::Plain { html } = &n.kind {
                            r.sys.legend_html = Some(html.clone());
                            r.legend_ref_id = Some(n.id.clone());
                        }
                    }
                }
            }
            Stmt::Edge(e) => pending.push((e.src.clone(), e.dst.clone(), e.attrs.clone())),
        }
    }

    // Pass B: resolve edges. `resolve` returns (EndRef, is_info_port_gap):
    // an info-port endpoint has no EndRef and falls back to the whole node,
    // producing `nX -> …` where HS emits `nX:info -> …` (a clean-model gap).
    let resolve = |ep: &graph_clean::model::EndPoint, r: &Recon| -> Option<(EndRef, bool)> {
        if let Some(p) = &ep.port {
            if let Some(er) = r.port.get(p) {
                return Some((*er, false));
            }
            if r.info_port.contains_key(p) {
                // info-port anchored edge: clean EndRef cannot express it.
                return r.whole.get(&ep.node).map(|i| (EndRef::Whole(*i), true));
            }
            return r.whole.get(&ep.node).map(|i| (EndRef::Whole(*i), false));
        }
        r.whole.get(&ep.node).map(|i| (EndRef::Whole(*i), false))
    };

    for (src, dst, attrs) in &pending {
        let is_legend_target = r
            .legend_ref_id
            .as_deref()
            .map(|lid| dst.port.is_none() && dst.node == lid)
            .unwrap_or(false);
        if is_legend_target {
            if let Some((er, gap)) = resolve(src, &r) {
                if gap {
                    r.compromises.push("info-port-edge");
                }
                r.sys.legend_edges.push((er, ()));
            } else {
                r.compromises.push("legend-edge-src-unresolved");
            }
            continue;
        }
        let Some(style) = attrs_to_style(attrs) else {
            r.compromises.push("unknown-edge-style");
            continue;
        };
        let (Some((s, sg)), Some((d, dg))) = (resolve(src, &r), resolve(dst, &r)) else {
            r.compromises.push("edge-endpoint-unresolved");
            continue;
        };
        if sg || dg {
            r.compromises.push("info-port-edge");
        }
        r.sys.edges.push(SysEdge::new(s, d, style));
    }

    r
}

// ---------------------------------------------------------------------------
// Byte-divergence family classification (for reconstructable payloads).
// ---------------------------------------------------------------------------

fn first_diff_line(a: &str, b: &str) -> Option<(usize, String, String)> {
    let al: Vec<&str> = a.lines().collect();
    let bl: Vec<&str> = b.lines().collect();
    let n = al.len().min(bl.len());
    for i in 0..n {
        if al[i] != bl[i] {
            return Some((i, al[i].to_string(), bl[i].to_string()));
        }
    }
    if al.len() != bl.len() {
        let i = n;
        let ga = al.get(i).copied().unwrap_or("<EOF>").to_string();
        let gb = bl.get(i).copied().unwrap_or("<EOF>").to_string();
        return Some((i, ga, gb));
    }
    None
}

fn classify_line(line: &str) -> &'static str {
    if line.contains("shape=\"record\"") {
        "cell-fill-wrap" // record label: the wrapping/fill divergence
    } else if line.starts_with("nodesep=")
        || line.starts_with("ranksep=")
        || line.starts_with("node[")
        || line.starts_with("edge[")
        || line.starts_with("packmode=")
        || line.starts_with("splines=")
    {
        "header-dialect"
    } else if line.starts_with("subgraph \"cluster_") || line.starts_with("label=\"")
        || line.starts_with("fillcolor=")
    {
        "cluster-structure"
    } else if line.contains(" -> ") {
        "edge"
    } else if line.contains("shape=\"ellipse\"") || line.contains("shape=\"invtrapezium\"") {
        "ellipse-node"
    } else if line.contains("shape=\"plain\"") || line.contains("<TABLE") {
        "legend"
    } else {
        "other"
    }
}

fn main() {
    // Single-file mode: `fullgen_census <in.dot>` prints full-clean-generated DOT
    // to stdout (for the live-surface harness). Reconstruct -> generate -> to_dot.
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && args[1].ends_with(".dot") {
        let dot = std::fs::read_to_string(&args[1]).unwrap();
        let g = parse(&dot);
        let recon = reconstruct(&g);
        print!("{}", to_dot(&generate(&recon.sys)));
        eprintln!("compromises: {:?}", {
            let mut c = recon.compromises.clone();
            c.sort();
            c.dedup();
            c
        });
        return;
    }
    let dir = std::env::var("GRAPHCLEAN_CORPUS").expect("set GRAPHCLEAN_CORPUS");
    let out = std::env::var("FULLGEN_OUT").expect("set FULLGEN_OUT (clean payload dir)");
    std::fs::create_dir_all(&out).unwrap();
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .expect("read corpus dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "dot").unwrap_or(false))
        .collect();
    files.sort();

    let tsv_path = std::env::var("FULLGEN_TSV").ok();
    let mut tsv = String::new();

    let mut payloads = 0u64;
    let mut byte_exact = 0u64;
    let mut reconstructable = 0u64; // no compromises
    let mut recon_byte_exact = 0u64; // no compromises AND byte-exact
    // Non-edge byte fidelity: compare all lines EXCEPT edges (` -> `). Isolates
    // node/record/cluster/legend/header regeneration from the edge gaps, so it
    // ties full-generate back to the round-11 structure-reused fill ceiling.
    let mut nonedge_exact = 0u64;

    // family: for byte diffs among reconstructable payloads, keyed by first-diff line class
    let mut fam: HashMap<&'static str, u64> = HashMap::new();
    let mut fam_wit: HashMap<&'static str, (String, String, String)> = HashMap::new();
    // compromise families (adapter limits), keyed by reason
    let mut comp: HashMap<&'static str, u64> = HashMap::new();
    // payloads counted once per DISTINCT compromise reason present
    let mut comp_payloads: HashMap<&'static str, u64> = HashMap::new();

    for path in &files {
        let dot = std::fs::read_to_string(path).unwrap();
        payloads += 1;
        let g = parse(&dot);
        let recon = reconstruct(&g);
        let clean = to_dot(&generate(&recon.sys));

        // write clean payload for the semantic pass
        let fname = path.file_name().unwrap();
        std::fs::write(std::path::Path::new(&out).join(fname), &clean).unwrap();

        let ok = clean == dot;
        if ok {
            byte_exact += 1;
        }
        let strip_edges = |s: &str| -> String {
            s.lines().filter(|l| !l.contains(" -> ")).collect::<Vec<_>>().join("\n")
        };
        if strip_edges(&clean) == strip_edges(&dot) {
            nonedge_exact += 1;
        }
        if tsv_path.is_some() {
            let mut cs: Vec<&str> = recon.compromises.clone();
            cs.sort();
            cs.dedup();
            tsv.push_str(&format!(
                "{}\t{}\t{}\n",
                fname.to_string_lossy(),
                if ok { 1 } else { 0 },
                cs.join(",")
            ));
        }
        let clean_recon = recon.compromises.is_empty();
        if clean_recon {
            reconstructable += 1;
            if ok {
                recon_byte_exact += 1;
            } else {
                // classify the byte divergence
                if let Some((_, want, got)) = first_diff_line(&dot, &clean) {
                    let f = classify_line(&want);
                    *fam.entry(f).or_insert(0) += 1;
                    fam_wit.entry(f).or_insert((
                        path.file_name().unwrap().to_string_lossy().to_string(),
                        want,
                        got,
                    ));
                }
            }
        } else {
            let mut seen: std::collections::BTreeSet<&'static str> = Default::default();
            for c in &recon.compromises {
                *comp.entry(c).or_insert(0) += 1;
                seen.insert(c);
            }
            for c in seen {
                *comp_payloads.entry(c).or_insert(0) += 1;
            }
        }
    }

    let pct = |a: u64, b: u64| if b == 0 { 0.0 } else { 100.0 * a as f64 / b as f64 };
    println!("=== ROUND-12 FULL-GENERATE BYTE CENSUS over {} payloads ===", payloads);
    println!(
        "(a) WHOLE-PAYLOAD byte-exact (full clean pipeline): {}/{} = {:.3}%",
        byte_exact, payloads, pct(byte_exact, payloads)
    );
    println!(
        "    cleanly-reconstructable payloads (0 adapter compromises): {}/{} = {:.3}%",
        reconstructable, payloads, pct(reconstructable, payloads)
    );
    println!(
        "    byte-exact among cleanly-reconstructable:                 {}/{} = {:.3}%",
        recon_byte_exact, reconstructable, pct(recon_byte_exact, reconstructable)
    );
    println!(
        "    NON-EDGE byte-exact (node/record/cluster/legend/header):  {}/{} = {:.3}%",
        nonedge_exact, payloads, pct(nonedge_exact, payloads)
    );
    println!();
    println!("=== BYTE DIVERGENCE FAMILIES (cleanly-reconstructable payloads, first-diff line) ===");
    let mut fv: Vec<_> = fam.iter().collect();
    fv.sort_by(|a, b| b.1.cmp(a.1));
    for (f, n) in fv {
        println!("{:>7} x  {}", n, f);
        if let Some((file, want, got)) = fam_wit.get(*f) {
            let t = |s: &str| -> String {
                let s: String = s.chars().take(140).collect();
                s
            };
            println!("           witness file: {}", file);
            println!("           HS : {}", t(want));
            println!("           got: {}", t(got));
        }
    }
    println!();
    println!("=== ADAPTER-COMPROMISE FAMILIES (payloads generate() cannot structurally reproduce) ===");
    let mut cv: Vec<_> = comp_payloads.iter().collect();
    cv.sort_by(|a, b| b.1.cmp(a.1));
    for (c, n) in cv {
        let occ = comp.get(*c).copied().unwrap_or(0);
        println!("{:>7} payloads ({} occurrences)  {}", n, occ, c);
    }
    if let Some(p) = tsv_path {
        std::fs::write(p, tsv).unwrap();
    }
    // silence unused import warning for RuleInstance (kept for API documentation)
    let _ = std::mem::size_of::<RuleInstance>();
}

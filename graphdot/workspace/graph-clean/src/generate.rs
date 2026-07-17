//! Graph GENERATION (BEHAVIOR.md §6): mapping a proof-state / constraint-system
//! description onto the DOT [`Graph`] model, over an independent input model
//! designed from observed paired `main/proof` sequents and `interactive-graph-def`
//! graphs.
//!
//! What the observed system→graph mapping does, structurally (each point traces to
//! corpus/live observation — see BEHAVIOR.md §6):
//!   * a **rule instance** `#t : Rule[actions]` becomes a `record` node with up to
//!     three groups `{premises}|{info}|{conclusions}`; the **info** group (the
//!     `#t : Rule[…]` cell) is always present (100 % of 160 409 corpus records),
//!     and an **empty** premise or conclusion group is dropped (so a source rule
//!     with no premises renders `{info}|{concl}`);
//!   * an intruder-knowledge fact renders as a gray `!KU( m ) @ #t` **ellipse**;
//!     a protocol action/event as a darkblue `Fact @ #t` ellipse; a compressed
//!     intruder rule as an uncolored `#t : rule` ellipse;
//!   * an unresolved node referenced by an open premise renders as an
//!     **invtrapezium** `(#var, idx)` (BEHAVIOR.md §3d);
//!   * **edges** connect a conclusion port to a premise port (structural), or an
//!     intruder deduction (red dashed), message (gray30), temporal order (blue3 /
//!     black dashed), etc. — the finite observed style vocabulary (§3c);
//!   * `n<K>` ids come from [`crate::alloc`]; the header is inferred by role (§4).
//!
//! GAPS (need the GPL solver; not derivable from output): *which* nodes/edges a
//! raw constraint system yields (the compression content), the per-rule color
//! hash, and record-cell line wrapping. This layer takes node/edge lists, colors,
//! and pre-rendered cell text as INPUTS and assembles the exact DOT structure.

use crate::alloc::NodeIdAllocator;
use crate::model::*;
use crate::render::{escape_record, Fact};

/// A rule instance: one record node with premise / info / conclusion cells.
#[derive(Clone, Debug)]
pub struct RuleInstance {
    pub temporal: String,
    pub rule: String,
    pub role: String,
    pub premises: Vec<Fact>,
    pub actions: Vec<Fact>,
    pub conclusions: Vec<Fact>,
    /// Per-rule fill color (a solver-side hash — a GAP; supplied by the caller).
    pub fillcolor: String,
    /// `black` on the light MSR palette, `white` on the saturated role palette.
    pub fontcolor: String,
}

impl RuleInstance {
    pub fn new(temporal: &str, rule: &str, fillcolor: &str) -> Self {
        RuleInstance {
            temporal: temporal.into(),
            rule: rule.into(),
            role: Role::UNDEFINED.into(),
            premises: Vec::new(),
            actions: Vec::new(),
            conclusions: Vec::new(),
            fillcolor: fillcolor.into(),
            fontcolor: "black".into(),
        }
    }
    pub fn premises(mut self, f: Vec<Fact>) -> Self {
        self.premises = f;
        self
    }
    pub fn actions(mut self, f: Vec<Fact>) -> Self {
        self.actions = f;
        self
    }
    pub fn conclusions(mut self, f: Vec<Fact>) -> Self {
        self.conclusions = f;
        self
    }
    pub fn role(mut self, r: &str, fontcolor: &str) -> Self {
        self.role = r.into();
        self.fontcolor = fontcolor.into();
        self
    }
    fn cell_count(&self) -> usize {
        let p = self.premises.len();
        let c = self.conclusions.len();
        p + 1 /* info */ + c
    }
}

/// One node of the input system, in emission order.
#[derive(Clone, Debug)]
pub enum GraphNode {
    Rule(RuleInstance),
    /// `!KU( term ) @ #t` — intruder knowledge (drawn as a gray ellipse). `term`
    /// is the pre-rendered message text.
    Knowledge { term: String, temporal: String },
    /// `Fact @ #t` — a protocol action/event (darkblue ellipse). `fact` is
    /// pre-rendered.
    Action { fact: String, temporal: String },
    /// `#t : rule` — a compressed rule shown as a single uncolored ellipse.
    Compressed { temporal: String, rule: String },
    /// `(#var, idx)` — an unresolved node referenced by an open premise
    /// (invtrapezium).
    OpenTarget { node_var: String, premise_index: usize },
}

/// An endpoint reference into the input system (resolved to `n<K>[:n<port>]`).
#[derive(Clone, Copy, Debug)]
pub enum EndRef {
    /// The whole node (no port) — ellipses, invtrapezium, legend.
    Whole(usize),
    /// Conclusion `c` of rule node `n` (→ its conclusion port).
    Conclusion(usize, usize),
    /// Premise `p` of rule node `n` (→ its premise port).
    Premise(usize, usize),
}

/// The finite observed edge-style vocabulary (BEHAVIOR.md §3c). Each maps to a
/// fixed attribute list emitted verbatim.
#[derive(Clone, Copy, Debug)]
pub enum EdgeStyle {
    /// `style="bold",weight="10.0",color="gray50"` — structural, into a target.
    StructuralGray,
    /// `style="bold",weight="10.0"` — structural (uncolored).
    Structural,
    /// `color="gray30"` — message / standard edge.
    Message,
    /// `color="red",style="dashed"` — intruder `!KU` deduction.
    KnowledgeDeduction,
    /// `color="orangered2"` — deduction variant.
    Deduction,
    /// `color="blue3",style="dashed"` — temporal-order edge.
    TemporalBlue,
    /// `color="black",style="dashed"` — before / less-than temporal edge.
    TemporalBlack,
    /// `style="invis"` — ranking edge to the legend.
    Invis,
}

impl EdgeStyle {
    /// The exact attribute pairs, in observed key order.
    pub fn attrs(self) -> Vec<(String, String)> {
        let raw: &[(&str, &str)] = match self {
            EdgeStyle::StructuralGray => {
                &[("style", "bold"), ("weight", "10.0"), ("color", "gray50")]
            }
            EdgeStyle::Structural => &[("style", "bold"), ("weight", "10.0")],
            EdgeStyle::Message => &[("color", "gray30")],
            EdgeStyle::KnowledgeDeduction => &[("color", "red"), ("style", "dashed")],
            EdgeStyle::Deduction => &[("color", "orangered2")],
            EdgeStyle::TemporalBlue => &[("color", "blue3"), ("style", "dashed")],
            EdgeStyle::TemporalBlack => &[("color", "black"), ("style", "dashed")],
            EdgeStyle::Invis => &[("style", "invis")],
        };
        raw.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }
}

/// An edge in the input system.
#[derive(Clone, Debug)]
pub struct SysEdge {
    pub src: EndRef,
    pub dst: EndRef,
    pub style: EdgeStyle,
}

impl SysEdge {
    pub fn new(src: EndRef, dst: EndRef, style: EdgeStyle) -> Self {
        SysEdge { src, dst, style }
    }
}

/// A proof-state graph description: nodes in emission order, edges, and an
/// optional pre-rendered legend (inner HTML of the `plain` node) plus the invis
/// edges wiring it in.
#[derive(Clone, Debug, Default)]
pub struct System {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<SysEdge>,
    /// Legend inner HTML (from [`crate::abbrev::Abbreviator::legend_html`]); if
    /// non-empty a `{ rank="sink"; … }` block is appended.
    pub legend_html: Option<String>,
    /// Invis edges to the legend node (emitted after the sink block).
    pub legend_edges: Vec<(EndRef, ())>,
}

/// Resolved ids for one system node.
enum Resolved {
    // The info port is never an edge endpoint, so it is not retained here.
    Record { ports_prem: Vec<String>, ports_concl: Vec<String>, node: String },
    Node(String),
}

impl Resolved {
    fn node_id(&self) -> &str {
        match self {
            Resolved::Record { node, .. } => node,
            Resolved::Node(id) => id,
        }
    }
}

/// Build the DOT [`Graph`] for a [`System`]: allocate ids, emit nodes in order,
/// then edges, then the legend sink-block + its invis edges. Header is inferred
/// from roles.
pub fn generate(sys: &System) -> Graph {
    let mut alloc = NodeIdAllocator::new();
    let mut resolved: Vec<Resolved> = Vec::with_capacity(sys.nodes.len());
    let mut g = Graph::new(Header::Simple);

    // Pass 1: allocate ids and emit node statements in emission order.
    for node in &sys.nodes {
        match node {
            GraphNode::Rule(r) => {
                let ids = alloc.record(r.cell_count());
                let mut it = ids.ports.into_iter();
                let ports_prem: Vec<String> = (0..r.premises.len()).map(|_| it.next().unwrap()).collect();
                let port_info = it.next().unwrap();
                let ports_concl: Vec<String> = (0..r.conclusions.len()).map(|_| it.next().unwrap()).collect();
                let rec = build_record(r, &ports_prem, &port_info, &ports_concl);
                g.push(Stmt::Node(Node::record(ids.node.clone(), rec)));
                resolved.push(Resolved::Record { ports_prem, ports_concl, node: ids.node });
            }
            GraphNode::Knowledge { term, temporal } => {
                let id = alloc.node();
                let label = format!("!KU( {} ) @ #{}", term, temporal);
                g.push(Stmt::Node(Node::ellipse(id.clone(), Ellipse::colored(label, "gray"))));
                resolved.push(Resolved::Node(id));
            }
            GraphNode::Action { fact, temporal } => {
                let id = alloc.node();
                let label = format!("{} @ #{}", fact, temporal);
                g.push(Stmt::Node(Node::ellipse(id.clone(), Ellipse::colored(label, "darkblue"))));
                resolved.push(Resolved::Node(id));
            }
            GraphNode::Compressed { temporal, rule } => {
                let id = alloc.node();
                let label = format!("#{} : {}", temporal, rule);
                g.push(Stmt::Node(Node::ellipse(id.clone(), Ellipse::new(label))));
                resolved.push(Resolved::Node(id));
            }
            GraphNode::OpenTarget { node_var, premise_index } => {
                let id = alloc.node();
                g.push(Stmt::Node(Node::shaped(
                    id.clone(),
                    Shaped::invtrapezium(node_var, *premise_index),
                )));
                resolved.push(Resolved::Node(id));
            }
        }
    }

    // Pass 2: edges.
    for e in &sys.edges {
        let src = endpoint(&resolved, e.src);
        let dst = endpoint(&resolved, e.dst);
        g.push(Stmt::Edge(Edge { src, dst, attrs: e.style.attrs() }));
    }

    // Pass 3: legend sink-block, then its invis edges (observed order).
    if let Some(html) = &sys.legend_html {
        if !html.is_empty() {
            let legend_id = alloc.node();
            let block = RankBlock {
                rank: "sink".into(),
                body: vec![Stmt::Node(Node::plain(legend_id.clone(), html.clone()))],
            };
            g.push(Stmt::RankBlock(block));
            for (from, ()) in &sys.legend_edges {
                let src = endpoint(&resolved, *from);
                g.push(Stmt::Edge(Edge {
                    src,
                    dst: EndPoint::node(legend_id.clone()),
                    attrs: EdgeStyle::Invis.attrs(),
                }));
            }
        }
    }

    g.set_inferred_header();
    g
}

/// Assemble a record node's model from a rule instance and its allocated ports.
/// Empty premise / conclusion groups are dropped; the info group is always kept
/// (matches the observed group structure).
fn build_record(r: &RuleInstance, ports_prem: &[String], port_info: &str, ports_concl: &[String]) -> Record {
    let mut columns: Vec<Vec<Cell>> = Vec::new();
    if !r.premises.is_empty() {
        columns.push(cells(&r.premises, ports_prem));
    }
    let info_text = crate::render::render_info(&r.temporal, &r.rule, &r.actions);
    columns.push(vec![Cell::new(port_info, escape_record(&info_text))]);
    if !r.conclusions.is_empty() {
        columns.push(cells(&r.conclusions, ports_concl));
    }
    Record {
        columns,
        fillcolor: r.fillcolor.clone(),
        fontcolor: r.fontcolor.clone(),
        role: Role(r.role.clone()),
    }
}

fn cells(facts: &[Fact], ports: &[String]) -> Vec<Cell> {
    facts
        .iter()
        .zip(ports)
        .map(|(f, p)| Cell::new(p.clone(), escape_record(&f.render_flat())))
        .collect()
}

/// Resolve an [`EndRef`] to a serializer [`EndPoint`] using the id/port map.
fn endpoint(resolved: &[Resolved], r: EndRef) -> EndPoint {
    match r {
        EndRef::Whole(n) => EndPoint::node(resolved[n].node_id().to_string()),
        EndRef::Conclusion(n, c) => {
            if let Resolved::Record { node, ports_concl, .. } = &resolved[n] {
                EndPoint::port(node.clone(), ports_concl[c].clone())
            } else {
                EndPoint::node(resolved[n].node_id().to_string())
            }
        }
        EndRef::Premise(n, p) => {
            if let Resolved::Record { node, ports_prem, .. } = &resolved[n] {
                EndPoint::port(node.clone(), ports_prem[p].clone())
            } else {
                EndPoint::node(resolved[n].node_id().to_string())
            }
        }
    }
}

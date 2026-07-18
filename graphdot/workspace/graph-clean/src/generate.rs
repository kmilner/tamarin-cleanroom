//! Graph GENERATION (BEHAVIOR.md §4, §6): mapping a proof-state / constraint-system
//! description onto the DOT [`Graph`] model, over an independent input model
//! designed from observed paired `main/proof` sequents and `interactive-graph-def`
//! graphs.
//!
//! What the observed system→graph mapping does, structurally (each point traces to
//! corpus/live observation — see BEHAVIOR.md §4, §6):
//!   * a **rule instance** `#t : Rule[actions]` becomes a `record` node with up to
//!     three groups `{premises}|{info}|{conclusions}`; the **info** group (the
//!     `#t : Rule[…]` cell) is always present (100 % of 160 409 corpus records),
//!     and an **empty** premise or conclusion group is dropped (so a source rule
//!     with no premises renders `{info}|{concl}`);
//!   * an intruder-knowledge fact renders as a gray `!KU( m ) @ #t` **ellipse**;
//!     a protocol action/event as a darkblue `Fact @ #t` ellipse; a compressed
//!     intruder rule as an uncolored `#t : rule` ellipse; a bare **timepoint**
//!     (`#i`, `#decrypt`, the designated `#last`) as an uncolored `#var` ellipse;
//!   * an unresolved node referenced by an open premise renders as an
//!     **invtrapezium** `(#var, idx)` (BEHAVIOR.md §3d);
//!   * a **role**-annotated record (role ≠ `Undefined`) is packed into a
//!     `subgraph "cluster_<Role>_Session_<k>"` block; role-annotated theories emit
//!     the compact header and free (non-role) nodes stay at the top level, in id
//!     order, *before* every cluster (BEHAVIOR.md §4);
//!   * **edges** connect a conclusion port to a premise port (structural), or an
//!     intruder deduction (red dashed), message (gray30), temporal order (blue3 /
//!     black dashed), etc. — the finite observed style vocabulary (§3c);
//!   * `n<K>` ids come from [`crate::alloc`]; record-cell text is wrapped/escaped
//!     by [`crate::doclayout::wrap_cell_dot`]; the header is inferred by role (§4).
//!
//! Two cell-content input paths, both flowing through the same wrap/escape
//! pipeline: [`RuleInstance`] carries [`Fact`]s (this crate renders + wraps them),
//! and [`RawRule`] carries PRE-RENDERED cell strings (the consumer's own printer
//! produced them; this crate still wraps and escapes). GAPS (need the GPL solver;
//! not derivable from output): *which* nodes/edges/clusters a raw constraint system
//! yields, the per-rule/per-cluster color hash, and the accumulated-column wrap
//! trigger for cells deep on a record line (BEHAVIOR.md §3f).

use crate::alloc::NodeIdAllocator;
use crate::doclayout::{wrap_cell_dot, FILL_WIDTH};
use crate::model::*;
use crate::render::{render_info, Fact};

/// The per-cell minimum wrap budget (BEHAVIOR.md §3f): inside a record group a
/// cell's shared budget never drops below this floor — a cell whose flat
/// rendering is at most this many columns never wraps, however wide its siblings
/// are (live probe: a sibling forced far past the budget still leaves the target
/// fitting at flat ≤ 20, wrapping at 21).
const MIN_CELL_BUDGET: usize = 20;

/// A per-record cluster assignment (BEHAVIOR.md §4). `label` is the cluster label
/// WITHOUT the `cluster_` prefix (e.g. `Initiator_Session_1`, observed always
/// `<Role>_Session_<k>`); `color` is the 8-hex ARGB used for the block's
/// `color`/`fillcolor`. Both are solver-supplied (a content GAP).
#[derive(Clone, Debug)]
pub struct ClusterRef {
    pub label: String,
    pub color: String,
}

impl ClusterRef {
    pub fn new(label: &str, color: &str) -> Self {
        ClusterRef { label: label.into(), color: color.into() }
    }
}

/// A rule instance: one record node with premise / info / conclusion cells, built
/// from [`Fact`]s (the crate renders and wraps them).
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
    /// `black` on the light MSR palette, `white`/`black` on the role palette.
    pub fontcolor: String,
    /// If set, this record is packed into a `cluster_<label>` subgraph (§4).
    pub cluster: Option<ClusterRef>,
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
            cluster: None,
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
    /// Pack this record into the `cluster_<label>` subgraph (BEHAVIOR.md §4).
    pub fn cluster(mut self, label: &str, color: &str) -> Self {
        self.cluster = Some(ClusterRef::new(label, color));
        self
    }
    /// The flat, un-escaped cell strings (premises, info, conclusions), for the
    /// shared record builder.
    fn spec(&self) -> RecordSpec {
        RecordSpec {
            premises: self.premises.iter().map(Fact::render_flat).collect(),
            info: render_info(&self.temporal, &self.rule, &self.actions),
            conclusions: self.conclusions.iter().map(Fact::render_flat).collect(),
            prem_widths: Vec::new(),
            concl_widths: Vec::new(),
            fillcolor: self.fillcolor.clone(),
            fontcolor: self.fontcolor.clone(),
            role: self.role.clone(),
            cluster: self.cluster.clone(),
        }
    }
}

/// A rule instance whose cells are supplied as **pre-rendered flat strings**
/// (BEHAVIOR.md interop, round 5). The consumer renders fact/term text with its own
/// printer — including any abbreviation — and this crate's wrap + escape pipeline
/// ([`crate::doclayout::wrap_cell_dot`]) applies to those strings exactly as it does
/// to a [`RuleInstance`]'s rendered facts. `info` is the whole info-cell text
/// (`#t : Rule[…]`); `premises` and `conclusions` are one flat fact string per cell.
#[derive(Clone, Debug, Default)]
pub struct RawRule {
    pub premises: Vec<String>,
    pub info: String,
    pub conclusions: Vec<String>,
    /// Optional caller-supplied width inputs, one per premise cell (empty =
    /// derive everything from the display text). See [`CellWidths`].
    pub premise_widths: Vec<Option<CellWidths>>,
    /// Optional caller-supplied width inputs, one per conclusion cell.
    pub conclusion_widths: Vec<Option<CellWidths>>,
    pub fillcolor: String,
    pub fontcolor: String,
    pub role: String,
    pub cluster: Option<ClusterRef>,
}

impl RawRule {
    /// A raw record with the given info-cell text and per-rule fill color; role
    /// defaults to `Undefined`, fontcolor to `black`.
    pub fn new(info: &str, fillcolor: &str) -> Self {
        RawRule {
            premises: Vec::new(),
            info: info.into(),
            conclusions: Vec::new(),
            premise_widths: Vec::new(),
            conclusion_widths: Vec::new(),
            fillcolor: fillcolor.into(),
            fontcolor: "black".into(),
            role: Role::UNDEFINED.into(),
            cluster: None,
        }
    }
    pub fn premises(mut self, cells: Vec<String>) -> Self {
        self.premises = cells;
        self
    }
    pub fn conclusions(mut self, cells: Vec<String>) -> Self {
        self.conclusions = cells;
        self
    }
    /// Supply per-cell width inputs for the premise group (one entry per
    /// premise cell; `None` entries fall back to display-text estimates).
    pub fn premise_widths(mut self, w: Vec<Option<CellWidths>>) -> Self {
        self.premise_widths = w;
        self
    }
    /// Supply per-cell width inputs for the conclusion group.
    pub fn conclusion_widths(mut self, w: Vec<Option<CellWidths>>) -> Self {
        self.conclusion_widths = w;
        self
    }
    pub fn role(mut self, r: &str, fontcolor: &str) -> Self {
        self.role = r.into();
        self.fontcolor = fontcolor.into();
        self
    }
    pub fn cluster(mut self, label: &str, color: &str) -> Self {
        self.cluster = Some(ClusterRef::new(label, color));
        self
    }
    fn spec(&self) -> RecordSpec {
        RecordSpec {
            premises: self.premises.clone(),
            info: self.info.clone(),
            conclusions: self.conclusions.clone(),
            prem_widths: self.premise_widths.clone(),
            concl_widths: self.conclusion_widths.clone(),
            fillcolor: self.fillcolor.clone(),
            fontcolor: self.fontcolor.clone(),
            role: self.role.clone(),
            cluster: self.cluster.clone(),
        }
    }
}

/// Caller-supplied per-cell width inputs, overriding the shape-feature
/// estimates [`group_widths`] derives from the cell's (post-abbreviation)
/// display text. The reference decides row sharing on its *internal*
/// (UN-abbreviated) term widths, which are structurally invisible to a crate
/// consuming display text — a caller that knows them (e.g. an adapter sitting
/// on the term representation) can pass them here (BEHAVIOR.md §3f, round 10).
/// Every field is optional; an absent field (or an absent [`CellWidths`]
/// altogether) falls back to the display-text estimate, byte-identically to
/// the no-override path.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CellWidths {
    /// The cell's row **occupancy** `C` (columns it counts for in its
    /// siblings' trigger budgets). Display-text default:
    /// `flat + Σ_{top-level tuple/union args}(elems + 1)`.
    pub occupancy: Option<i64>,
    /// The cell's own trigger-budget **bonus**. Display-text default: the
    /// largest `⌊elems/2⌋ + 2` over its top-level tuple/union args (4 for an
    /// arg with ≥ 9 elements), 0 without such an arg.
    pub bonus: Option<i64>,
    /// The cell's **fill numerator** (its internal width in the proportional
    /// fill share once it wraps). Display-text default:
    /// `flat + Σ_{union args}(elems + 1) + #function-nodes`.
    pub fill_width: Option<i64>,
}

impl CellWidths {
    /// An override that fixes only the row occupancy.
    pub fn occupancy(c: i64) -> Self {
        CellWidths { occupancy: Some(c), ..Default::default() }
    }
}

/// The flat (un-escaped, un-wrapped) content of a record, shared by the Term-based
/// [`RuleInstance`] and the pre-rendered [`RawRule`]. Cell text is wrapped and
/// escaped by [`build_record`]. `prem_widths` / `concl_widths`, when non-empty,
/// carry one optional [`CellWidths`] per premise / conclusion cell.
struct RecordSpec {
    premises: Vec<String>,
    info: String,
    conclusions: Vec<String>,
    prem_widths: Vec<Option<CellWidths>>,
    concl_widths: Vec<Option<CellWidths>>,
    fillcolor: String,
    fontcolor: String,
    role: String,
    cluster: Option<ClusterRef>,
}

impl RecordSpec {
    fn cell_count(&self) -> usize {
        self.premises.len() + 1 /* info */ + self.conclusions.len()
    }
}

/// One node of the input system, in emission order.
#[derive(Clone, Debug)]
pub enum GraphNode {
    /// A rule instance built from [`Fact`]s (rendered + wrapped by this crate).
    Rule(RuleInstance),
    /// A rule instance whose cells are pre-rendered strings (still wrapped/escaped).
    RawRule(RawRule),
    /// `!KU( term ) @ #t` — intruder knowledge (drawn as a gray ellipse). `term`
    /// is the pre-rendered message text.
    Knowledge { term: String, temporal: String },
    /// `Fact @ #t` — a protocol action/event (darkblue ellipse). `fact` is
    /// pre-rendered.
    Action { fact: String, temporal: String },
    /// `#t : rule` — a compressed rule shown as a single uncolored ellipse.
    Compressed { temporal: String, rule: String },
    /// `#var` — a bare timepoint ellipse (uncolored). Observed for ordinary
    /// timepoint variables (`#i`, `#decrypt`, `#t1`, …) and, when a constraint
    /// system carries a designated last timepoint (induction), the `#last` node
    /// (`#last` is the target of `color="black",style="dashed"` before-edges).
    Temporal { var: String },
    /// `(#var, idx)` — an unresolved node referenced by an open premise
    /// (invtrapezium), the target of a conclusion→absent-node structural edge.
    OpenTarget { node_var: String, premise_index: usize },
    /// A node with an explicit `shape` and label — the extension point for shapes
    /// beyond the observed set (e.g. the `trapezium` dual, an unresolved source
    /// feeding a present premise, which is spec-named but was **not observed** in
    /// the corpus or any probe; see BEHAVIOR.md §3d).
    Shaped { label: String, shape: String, color: Option<String> },
}

impl GraphNode {
    /// The designated last timepoint `#last` (BEHAVIOR.md §3d/§6).
    pub fn last() -> Self {
        GraphNode::Temporal { var: "last".into() }
    }
    /// A record node from a [`RecordSpec`], if this node is one.
    fn record_spec(&self) -> Option<RecordSpec> {
        match self {
            GraphNode::Rule(r) => Some(r.spec()),
            GraphNode::RawRule(r) => Some(r.spec()),
            _ => None,
        }
    }
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
    /// `color="black",style="dashed"` — before / less-than temporal edge (e.g. into
    /// the `#last` node).
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

/// Build the DOT [`Graph`] for a [`System`]: allocate ids in emission order (so id
/// order == file order), emit free (non-clustered) node statements at the top
/// level, then the `cluster_…` subgraph blocks (role records, grouped by cluster
/// label in first-appearance order), then edges, then the legend sink-block + its
/// invis edges. Header is inferred from roles (§4).
pub fn generate(sys: &System) -> Graph {
    let mut alloc = NodeIdAllocator::new();
    let mut resolved: Vec<Resolved> = Vec::with_capacity(sys.nodes.len());

    // Free top-level node statements, in emission order.
    let mut free: Vec<Stmt> = Vec::new();
    // Clusters: label -> (color, record statements), with first-appearance order.
    let mut cluster_order: Vec<String> = Vec::new();
    let mut clusters: std::collections::HashMap<String, (String, Vec<Stmt>)> =
        std::collections::HashMap::new();

    // Pass 1: allocate ids and build node statements in emission order. A clustered
    // record's statement is routed into its cluster bucket; everything else is free.
    for node in &sys.nodes {
        if let Some(spec) = node.record_spec() {
            let ids = alloc.record(spec.cell_count());
            let mut it = ids.ports.into_iter();
            let ports_prem: Vec<String> = (0..spec.premises.len()).map(|_| it.next().unwrap()).collect();
            let port_info = it.next().unwrap();
            let ports_concl: Vec<String> =
                (0..spec.conclusions.len()).map(|_| it.next().unwrap()).collect();
            let rec = build_record(&spec, &ports_prem, &port_info, &ports_concl);
            let stmt = Stmt::Node(Node::record(ids.node.clone(), rec));
            match &spec.cluster {
                Some(c) => {
                    let entry = clusters.entry(c.label.clone()).or_insert_with(|| {
                        cluster_order.push(c.label.clone());
                        (c.color.clone(), Vec::new())
                    });
                    entry.1.push(stmt);
                }
                None => free.push(stmt),
            }
            resolved.push(Resolved::Record { ports_prem, ports_concl, node: ids.node });
            continue;
        }
        let id = alloc.node();
        let stmt = match node {
            GraphNode::Knowledge { term, temporal } => {
                let label = format!("!KU( {} ) @ #{}", term, temporal);
                Stmt::Node(Node::ellipse(id.clone(), Ellipse::colored(label, "gray")))
            }
            GraphNode::Action { fact, temporal } => {
                let label = format!("{} @ #{}", fact, temporal);
                Stmt::Node(Node::ellipse(id.clone(), Ellipse::colored(label, "darkblue")))
            }
            GraphNode::Compressed { temporal, rule } => {
                let label = format!("#{} : {}", temporal, rule);
                Stmt::Node(Node::ellipse(id.clone(), Ellipse::new(label)))
            }
            GraphNode::Temporal { var } => {
                Stmt::Node(Node::ellipse(id.clone(), Ellipse::new(format!("#{}", var))))
            }
            GraphNode::OpenTarget { node_var, premise_index } => {
                Stmt::Node(Node::shaped(id.clone(), Shaped::invtrapezium(node_var, *premise_index)))
            }
            GraphNode::Shaped { label, shape, color } => Stmt::Node(Node::shaped(
                id.clone(),
                Shaped { label: label.clone(), shape: shape.clone(), color: color.clone() },
            )),
            GraphNode::Rule(_) | GraphNode::RawRule(_) => unreachable!("records handled above"),
        };
        free.push(stmt);
        resolved.push(Resolved::Node(id));
    }

    // Assemble: free nodes, then clusters (first-appearance order), then edges.
    let mut g = Graph::new(Header::Simple);
    for s in free {
        g.push(s);
    }
    for label in &cluster_order {
        let (color, body) = clusters.remove(label).unwrap();
        g.push(Stmt::Cluster(Cluster { label: label.clone(), color, body }));
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

/// Assemble a record node's model from a [`RecordSpec`] and its allocated ports.
/// Empty premise / conclusion groups are dropped; the info group is always kept
/// (matches the observed group structure). Each prem/concl group's cells share the
/// record wrap budget by the proportional allocation [`group_widths`]; the info
/// cell is its own single-cell group (budget 87). Cell text is wrapped (ragged
/// HughesPJ `fill`) and escaped by [`crate::doclayout::wrap_cell_dot`].
fn build_record(spec: &RecordSpec, ports_prem: &[String], port_info: &str, ports_concl: &[String]) -> Record {
    let mut columns: Vec<Vec<Cell>> = Vec::new();
    if !spec.premises.is_empty() {
        columns.push(group_cells(&spec.premises, ports_prem, &spec.prem_widths));
    }
    // The info cell is its own single-cell group (full width), fed through the
    // faithful layout engine at [`FILL_WIDTH`].
    columns.push(vec![Cell::new(port_info, wrap_cell_dot(&spec.info, FILL_WIDTH as isize))]);
    if !spec.conclusions.is_empty() {
        columns.push(group_cells(&spec.conclusions, ports_concl, &spec.concl_widths));
    }
    Record {
        columns,
        fillcolor: spec.fillcolor.clone(),
        fontcolor: spec.fontcolor.clone(),
        role: Role(spec.role.clone()),
    }
}

/// Shape features of one cell's flat text that enter the row-share model
/// (probe-derived, BEHAVIOR.md §3f rounds 9–10): `flat` = display width;
/// `tup_sur` / `uni_sur` = Σ over top-level tuple / `(a++b)`-union arguments
/// of `elems + 1` (the round-10 occupancy law, pinned at n = 2,3,4,6 tuples
/// and 3,5,8 unions); `bmax` = the largest per-argument self-budget bonus
/// `⌊elems/2⌋ + 2` (arguments with ≥ 9 elements contribute 4); `sqa` = the
/// cell is a fact with exactly one argument that is a single-quoted constant
/// (enters only the fill weight); `nfunc` = number of function-application
/// nodes anywhere in the text (enters only the fill numerator).
struct CellShape {
    flat: usize,
    tup_sur: i64,
    uni_sur: i64,
    bmax: i64,
    sqa: bool,
    nfunc: i64,
}

/// Split a term list at top-level `", "`, honoring nesting and quotes.
fn split_level(s: &str) -> Vec<&str> {
    let b: Vec<(usize, char)> = s.char_indices().collect();
    let mut parts = Vec::new();
    let (mut depth, mut inq, mut start) = (0i32, false, 0usize);
    let mut i = 0;
    while i < b.len() {
        let (pos, c) = b[i];
        if inq {
            if c == '\'' {
                inq = false;
            }
        } else {
            match c {
                '\'' => inq = true,
                '(' | '<' | '[' => depth += 1,
                ')' | '>' | ']' => depth -= 1,
                ',' if depth == 0 && i + 1 < b.len() && b[i + 1].1 == ' ' => {
                    parts.push(&s[start..pos]);
                    start = b[i + 1].0 + 1;
                    i += 1;
                }
                _ => {}
            }
        }
        i += 1;
    }
    parts.push(&s[start..]);
    parts
}

/// Element count of a parenthesized top-level `++`-union `(a++b++…)`, 0 for
/// anything else.
fn union_elems(t: &str) -> i64 {
    if !(t.starts_with('(') && t.ends_with(')')) || t.len() < 2 {
        return 0;
    }
    let inner = &t[1..t.len() - 1];
    let b: Vec<char> = inner.chars().collect();
    let (mut depth, mut inq, mut n) = (0i32, false, 1i64);
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if inq {
            if c == '\'' {
                inq = false;
            }
        } else {
            match c {
                '\'' => inq = true,
                '(' | '<' | '[' => depth += 1,
                ')' | '>' | ']' => depth -= 1,
                '+' if depth == 0 && i + 1 < b.len() && b[i + 1] == '+' => {
                    n += 1;
                    i += 1;
                }
                _ => {}
            }
        }
        i += 1;
    }
    if n >= 2 { n } else { 0 }
}

fn cell_shape(flat: &str) -> CellShape {
    let width = flat.chars().count();
    let (mut tup_sur, mut uni_sur, mut bmax) = (0i64, 0i64, 0i64);
    let mut sqa = false;
    if let Some(open) = flat.find("( ") {
        // a padded fact with at least one argument (`Name( )` has none)
        if flat.ends_with(" )")
            && open + 2 <= flat.len() - 2
            && !flat[..open].contains(['(', ')', '<', '>', ' ', ','])
        {
            let inner = &flat[open + 2..flat.len() - 2];
            let args = split_level(inner);
            for a in &args {
                let t = a.trim();
                let (elems, is_tuple) = if t.starts_with('<') && t.ends_with('>') {
                    (split_level(&t[1..t.len() - 1]).len() as i64, true)
                } else {
                    (union_elems(t), false)
                };
                if elems >= 2 {
                    if is_tuple {
                        tup_sur += elems + 1;
                    } else {
                        uni_sur += elems + 1;
                    }
                    bmax = bmax.max(if elems <= 8 { elems / 2 + 2 } else { 4 });
                }
            }
            if args.len() == 1 {
                let a = args[0].trim();
                sqa = a.starts_with('\'')
                    && a.ends_with('\'')
                    && a.len() >= 2
                    && !a[1..a.len() - 1].contains('\'');
            }
        }
    }
    // function-application nodes: identifier directly followed by `(` with no
    // space after (the unpadded display form), e.g. `senc(`, `pk(`
    let bch: Vec<char> = flat.chars().collect();
    let mut nfunc = 0i64;
    for i in 1..bch.len() {
        if bch[i] == '('
            && (bch[i - 1].is_alphanumeric() || bch[i - 1] == '_')
            && (i + 1 >= bch.len() || bch[i + 1] != ' ')
        {
            nfunc += 1;
        }
    }
    CellShape { flat: width, tup_sur, uni_sur, bmax, sqa, nfunc }
}

/// The per-cell fit **budgets** of one record group (all premises together, or
/// all conclusions together), from the cells' flat texts. Two probe-derived
/// layers (BEHAVIOR.md §3f, rounds 9–10; every parameter pinned by live probe
/// batteries — QUERIES.log Sessions 9–10):
///
/// **Trigger** (which cells wrap). Each cell occupies
/// `C_j = flat_j + Σ_{top-level tuple/union args}(elems + 1)` columns of the
/// row (the round-10 size law: X-flip boundaries at n = 2,3,4,6 tuples and
/// 3,5,8 unions are each exact to the column); cell *i*'s trigger budget is
/// `max(87 + bonus_i − Σ_{j≠i} C_j, 20)` in a multi-cell row, where
/// `bonus_i` = the largest `⌊elems/2⌋ + 2` over its tuple/union args (4 for
/// an arg with ≥ 9 elements — the r8 16/20-element rows cap it), and it
/// wraps iff its flat width exceeds that budget. A lone cell's budget is
/// exactly 87. There is NO single-quoted-atom, multi-quote, or
/// function-node correction (round-10 batteries A–C: every such sib flips
/// exactly at the plain-flat crossing). The only systematic residual is the
/// `[45-argfact partner, budget+1]` relief (9/731 probe cells, all one
/// pattern: beside a 45-flat argfact partner a cell at exactly budget+1
/// stays flat; beside 46 or in triples it does not) — the known ±1
/// coupled-`fits` flip, not modelable in closed form.
///
/// **Fill** (the ribbon a wrapping cell is laid out at):
/// `round(87·N_i / (N_i + Σ_{j≠i} w_j·flat_j))`, half-up, clamped to
/// `[20, flat_i − 1]`, where the numerator `N_i = flat_i + Σ_{union
/// args}(elems + 1) + nfunc_i` is the cell's *internal* width (unions carry
/// their spaced internal separators, function applications ~1 column each —
/// pinned by the UD/UG/DN/FE squeezed fills), and `w_j = 5/6` for
/// single-quoted-atom siblings of a tuple-fact receiver (round-9 Q/I series:
/// the [Big 87, atom s] fill follows 87²/(87 + 5s/6) across s = 12…120),
/// else 1 (round-10 argfact receivers track the plain share). Tuple args do
/// NOT enter the numerator (the live `Wide` record byte-pins the tuple
/// receiver at the display-flat share). A fitting cell keeps
/// `max(trigger, flat)` so it renders flat.
pub fn group_widths(cells: &[String]) -> Vec<usize> {
    group_widths_with(cells, &[])
}

/// [`group_widths`] with caller-supplied per-cell width inputs. `overrides` is
/// either empty (all cells use display-text estimates — byte-identical to
/// [`group_widths`]) or one `Option<CellWidths>` per cell; each present field
/// of a present entry replaces the corresponding display-text estimate
/// (occupancy `C`, budget bonus, fill numerator) for that cell, and every
/// absent field falls back per-field. The display flat width itself always
/// comes from the text (it *is* the rendered content).
pub fn group_widths_with(cells: &[String], overrides: &[Option<CellWidths>]) -> Vec<usize> {
    let shapes: Vec<CellShape> = cells.iter().map(|t| cell_shape(t)).collect();
    let n = shapes.len();
    let full = FILL_WIDTH as i64; // 87
    let ov = |i: usize| -> Option<&CellWidths> { overrides.get(i).and_then(|o| o.as_ref()) };
    let cs: Vec<i64> = shapes
        .iter()
        .enumerate()
        .map(|(i, s)| {
            ov(i)
                .and_then(|w| w.occupancy)
                .unwrap_or(s.flat as i64 + s.tup_sur + s.uni_sur)
        })
        .collect();
    let ctot: i64 = cs.iter().sum();
    let mut out = Vec::with_capacity(n);
    for (i, sh) in shapes.iter().enumerate() {
        let flat = sh.flat as i64;
        if n == 1 {
            out.push(full as usize);
            continue;
        }
        let bonus = ov(i).and_then(|w| w.bonus).unwrap_or(sh.bmax);
        let b_trig = (full + bonus - (ctot - cs[i])).max(MIN_CELL_BUDGET as i64);
        if flat <= b_trig {
            // fits: any budget ≥ flat renders the cell flat
            out.push(b_trig.max(flat) as usize);
            continue;
        }
        // wraps: proportional fill share; the numerator is the cell's internal
        // width (union separators + function nodes), siblings at their
        // occupancy C with single-quoted atoms discounted 5/6 for a
        // tuple-fact receiver
        let num = ov(i)
            .and_then(|w| w.fill_width)
            .unwrap_or(flat + sh.uni_sur + sh.nfunc) as f64;
        let mut t = num;
        for j in 0..n {
            if j != i {
                let w = if shapes[j].sqa && sh.tup_sur > 0 { 5.0 / 6.0 } else { 1.0 };
                t += w * cs[j] as f64;
            }
        }
        let mut b = ((full as f64) * num / t + 0.5).floor() as i64;
        b = b.max(MIN_CELL_BUDGET as i64).min((flat - 1).max(MIN_CELL_BUDGET as i64));
        out.push(b as usize);
    }
    out
}

/// Wrap every cell of one record group, sharing the row via
/// [`group_widths_with`] and laying each cell out at its budget with the
/// faithful engine.
fn group_cells(flat_cells: &[String], ports: &[String], widths: &[Option<CellWidths>]) -> Vec<Cell> {
    let fills = group_widths_with(flat_cells, widths);
    flat_cells
        .iter()
        .zip(ports)
        .zip(fills)
        .map(|((text, p), w)| Cell::new(p.clone(), wrap_cell_dot(text, w as isize)))
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

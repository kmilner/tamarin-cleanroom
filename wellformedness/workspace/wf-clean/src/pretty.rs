//! Term / fact pretty-printer reproducing the oracle's rendering.
//!
//! Every rule here traces to an oracle observation (see workspace/BEHAVIOR.md
//! and the probes t_terms, t_xor, t_nat, f_nullary).

use crate::ast::*;

/// Render a variable: sort prefix + base name + optional ".idx" (idx > 0).
pub fn pp_var(v: &VarSpec) -> String {
    let prefix = match v.sort {
        SortHint::Fresh => "~",
        SortHint::Pub => "$",
        SortHint::Nat => "%",
        SortHint::Node => "#",
        SortHint::Msg | SortHint::Untagged => "",
        // Suffix-sorted variables carry an explicit ":sort"; best-effort.
        SortHint::Suffix(_) => "",
    };
    if v.idx > 0 {
        format!("{}{}.{}", prefix, v.name, v.idx)
    } else {
        format!("{}{}", prefix, v.name)
    }
}

/// Flatten a right-nested pair term into its element list.
fn flatten_pair(t: &Term, out: &mut Vec<String>) {
    match t {
        Term::Pair(items) => {
            for it in items {
                flatten_pair(it, out);
            }
        }
        Term::App(name, args) if name == "pair" && args.len() == 2 => {
            flatten_pair(&args[0], out);
            flatten_pair(&args[1], out);
        }
        other => out.push(pp_term(other)),
    }
}

/// Collect the operands of a left/right-nested chain of the same binary op.
fn flatten_binop(op: BinOp, t: &Term, out: &mut Vec<Term>) {
    if let Term::BinOp(o, a, b) = t {
        if *o == op {
            flatten_binop(op, a, out);
            flatten_binop(op, b, out);
            return;
        }
    }
    out.push(t.clone());
}

fn pp_binop(op: BinOp, a: &Term, b: &Term) -> String {
    match op {
        // Exponentiation is shown infix without surrounding parentheses.
        BinOp::Exp => format!("{}^{}", pp_term(a), pp_term(b)),
        // The AC operators are parenthesised and joined by their symbol.
        BinOp::Mult | BinOp::Union | BinOp::Xor | BinOp::NatPlus => {
            let sym = match op {
                BinOp::Mult => "*",
                BinOp::Union => "++",
                BinOp::Xor => "\u{2295}", // (+)
                BinOp::NatPlus => "%+",
                BinOp::Exp => unreachable!(),
            };
            let whole = Term::BinOp(op, Box::new(a.clone()), Box::new(b.clone()));
            let mut operands = Vec::new();
            flatten_binop(op, &whole, &mut operands);
            let mut parts: Vec<String> = operands.iter().map(pp_term).collect();
            // XOR arguments are normalised (sorted); observed in probe t_xor.
            if op == BinOp::Xor {
                parts.sort();
            }
            format!("({})", parts.join(sym))
        }
    }
}

/// Render a term the way the oracle prints it inside facts.
pub fn pp_term(t: &Term) -> String {
    match t {
        Term::Var(v) => pp_var(v),
        Term::PubLit(s) => format!("'{}'", s),
        Term::FreshLit(s) => format!("~'{}'", s),
        Term::NatLit(s) => format!("%'{}'", s),
        Term::Number(n) => n.to_string(),
        Term::NumberOne => "1".to_string(),
        Term::NatOne => "%1".to_string(),
        Term::DhNeutral => "DH_neutral".to_string(),
        Term::App(name, args) => {
            if name == "pair" && args.len() == 2 {
                let mut elems = Vec::new();
                flatten_pair(t, &mut elems);
                format!("<{}>", elems.join(", "))
            } else if args.is_empty() {
                name.clone()
            } else {
                let parts: Vec<String> = args.iter().map(pp_term).collect();
                format!("{}({})", name, parts.join(", "))
            }
        }
        Term::AlgApp(name, a, b) => format!("{}({}, {})", name, pp_term(a), pp_term(b)),
        Term::Pair(_) => {
            let mut elems = Vec::new();
            flatten_pair(t, &mut elems);
            format!("<{}>", elems.join(", "))
        }
        Term::Diff(a, b) => format!("diff({}, {})", pp_term(a), pp_term(b)),
        Term::BinOp(op, a, b) => pp_binop(*op, a, b),
        Term::PatMatch(inner) => format!("={}", pp_term(inner)),
    }
}

/// Render a fact: optional `!` for persistent, then `Name( args )`.
/// Empty argument list renders as `Name( )`.
pub fn pp_fact(f: &Fact) -> String {
    let bang = if f.persistent { "!" } else { "" };
    if f.args.is_empty() {
        format!("{}{}( )", bang, f.name)
    } else {
        let parts: Vec<String> = f.args.iter().map(pp_term).collect();
        format!("{}{}( {} )", bang, f.name, parts.join(", "))
    }
}

/// Render a bracketed fact list as it appears inside a rule: `[ f1, f2 ]`, or
/// `[ ]` when empty.
pub fn pp_fact_list(fs: &[Fact]) -> String {
    if fs.is_empty() {
        "[ ]".to_string()
    } else {
        let parts: Vec<String> = fs.iter().map(pp_fact).collect();
        format!("[ {} ]", parts.join(", "))
    }
}

// ---------------------------------------------------------------------------
// Equation layout (Subterm Convergence Warning entries)
// ---------------------------------------------------------------------------
// Calibrated against the oracle (probes t5_wl*/t5_tup*/t5_last* and the
// ble/mesh reference blocks). An equation renders flat as `    lhs = rhs`
// while it fits; otherwise the LHS keeps the 4-column indent and the RHS
// continues on the next line as `  = rhs`. Inside the RHS, function
// applications fill their comma-glued arguments at the column after the
// opening paren, tuples fill their `", "`-suffixed elements one column after
// the `<` (a line then ends with the trailing `", "`), and every fit check
// compares against min(100, 67 + nest of the current line) - the nest being
// the continuation column that started the line. A tuple element following a
// multi-line element always starts a fresh line, and the closing `>` drops to
// the tuple's start column when the last element was multi-line (or when it
// does not fit); an application's `)` always glues.

fn eq_margin(line_nest: usize) -> usize {
    std::cmp::min(100, line_nest + 67)
}

struct EqLayout {
    out: String,
    col: usize,
    line_nest: usize,
}

impl EqLayout {
    fn emit(&mut self, s: &str) {
        self.out.push_str(s);
        self.col += s.chars().count();
    }
    fn newline(&mut self, nest: usize) {
        self.out.push('\n');
        self.out.push_str(&" ".repeat(nest));
        self.col = nest;
        self.line_nest = nest;
    }
    fn margin(&self) -> usize {
        eq_margin(self.line_nest)
    }
}

/// The flattened element list of a tuple term, or `None` when `t` is not a
/// tuple (mirrors the flat printer's pair flattening).
fn tuple_elems(t: &Term) -> Option<Vec<Term>> {
    match t {
        Term::Pair(_) => (),
        Term::App(name, args) if name == "pair" && args.len() == 2 => (),
        _ => return None,
    }
    let mut parts = Vec::new();
    fn walk(t: &Term, out: &mut Vec<Term>) {
        match t {
            Term::Pair(items) => {
                for it in items {
                    walk(it, out);
                }
            }
            Term::App(name, args) if name == "pair" && args.len() == 2 => {
                walk(&args[0], out);
                walk(&args[1], out);
            }
            other => out.push(other.clone()),
        }
    }
    walk(t, &mut parts);
    Some(parts)
}

/// Lay a term at the current position, breaking when it exceeds the margin.
/// Returns whether the layout spanned multiple lines.
fn lay_eq_term(t: &Term, st: &mut EqLayout) -> bool {
    let flat = pp_term(t);
    if st.col + flat.chars().count() <= st.margin() {
        st.emit(&flat);
        return false;
    }
    if let Some(elems) = tuple_elems(t) {
        let start_col = st.col;
        st.emit("<");
        let cont = st.col;
        let mut prev_multiline = false;
        for (i, e) in elems.iter().enumerate() {
            let sep = if i + 1 < elems.len() { ", " } else { "" };
            let flat_e = pp_term(e);
            let w = flat_e.chars().count() + sep.chars().count();
            if !prev_multiline && st.col + w <= st.margin() {
                st.emit(&flat_e);
                st.emit(sep);
            } else {
                st.newline(cont);
                prev_multiline = lay_eq_term(e, st);
                st.emit(sep);
            }
        }
        if prev_multiline || st.col + 1 > st.margin() {
            st.newline(start_col);
        }
        st.emit(">");
        return true;
    }
    match t {
        Term::App(name, args) if !args.is_empty() => {
            st.emit(name);
            st.emit("(");
            let cont = st.col;
            let mut first_on_line = true;
            for (i, a) in args.iter().enumerate() {
                let sep = if i + 1 < args.len() { "," } else { "" };
                let flat_a = pp_term(a);
                let lead = if first_on_line { 0 } else { 1 };
                let w = lead + flat_a.chars().count() + sep.chars().count();
                if st.col + w <= st.margin() {
                    if !first_on_line {
                        st.emit(" ");
                    }
                    st.emit(&flat_a);
                    st.emit(sep);
                    first_on_line = false;
                } else {
                    st.newline(cont);
                    lay_eq_term(a, st);
                    st.emit(sep);
                    first_on_line = false;
                }
            }
            st.emit(")");
        }
        // No other break points: emit flat (over-wide atoms overflow).
        _ => st.emit(&flat),
    }
    true
}

/// Render one flagged equation for the Subterm Convergence Warning block:
/// `    lhs = rhs` on one line while it fits within column 71, else the
/// wrapped form with `  = ` starting the RHS.
pub fn pp_equation(lhs: &Term, rhs: &Term) -> String {
    let flat = format!("    {} = {}", pp_term(lhs), pp_term(rhs));
    if flat.chars().count() <= eq_margin(4) {
        return flat;
    }
    let mut st = EqLayout {
        out: format!("    {}\n  = ", pp_term(lhs)),
        col: 4,
        line_nest: 2,
    };
    lay_eq_term(rhs, &mut st);
    st.out
}

/// Render a rule the way the oracle prints it (single-line body; the oracle
/// wraps very long rules across several lines - see BEHAVIOR.md gaps):
///
/// ```text
/// rule (modulo E) Name:
///    [ prems ] --> [ concls ]
/// ```
///
/// with `--[ acts ]->` in place of `-->` when the rule has action facts.
pub fn pp_rule(r: &Rule) -> String {
    let modulo = r.modulo.as_deref().unwrap_or("E");
    let prems = pp_fact_list(&r.premises);
    let concls = pp_fact_list(&r.conclusions);
    let arrow = if r.actions.is_empty() {
        "-->".to_string()
    } else {
        let acts: Vec<String> = r.actions.iter().map(pp_fact).collect();
        format!("--[ {} ]->", acts.join(", "))
    };
    format!("rule (modulo {}) {}:\n   {} {} {}", modulo, r.name, prems, arrow, concls)
}

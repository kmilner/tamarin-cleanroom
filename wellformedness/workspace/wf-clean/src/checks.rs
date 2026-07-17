//! The individual wellformedness checks. Each produces zero or one `WfError`
//! (one topic block). Message bodies reproduce oracle byte formatting.

use crate::ast::*;
use crate::pretty::{pp_fact, pp_rule, pp_term, pp_var};
use crate::report::WfError;

// ---- Topic strings (exact, including significant whitespace) ----
pub const T_UNBOUND: &str = "Unbound variables";
pub const T_FRESH_PUB: &str = "Fresh public constants";
pub const T_PUBNAMES: &str = "Public constants with mismatching capitalization";
pub const T_SORTS: &str = "Variable with mismatching sorts or capitalization";
pub const T_RESERVED: &str = "Reserved names";
pub const T_RESERVED_PREFIX: &str = "Reserved prefixes";
pub const T_FR: &str = "Fr facts must only use a fresh- or a msg-variable";
pub const T_SPECIAL: &str = "Special facts";
pub const T_FACT_CAP: &str = "Fact capitalization issues";
pub const T_ARITY: &str = "Fact arity issues";
pub const T_MULT: &str = "Fact multiplicity issues";
pub const T_LHSRHS: &str = "Facts occur in the left-hand-side but not in any right-hand-side ";
pub const T_LEFT: &str = "Left rule";
pub const T_RIGHT: &str = "Right rule";
pub const T_FORMULA_TERMS: &str = "Formula terms";
pub const T_GUARD: &str = " Formula guardedness";
pub const T_LEMMA_ANNOT: &str = "Lemma annotations";
pub const T_MULRESTRICT: &str = "Multiplication restriction of rules";
pub const T_NAT: &str = "Nat Sorts";
pub const T_SUBTERM: &str = "Subterm Convergence Warning";

/// Fact-name prefixes reserved for the diff-mode translation (observed in the
/// "Reserved prefixes" check, diff mode only).
const RESERVED_PREFIXES: &[&str] = &["DiffIntr", "DiffProto"];

/// Word-fill width used by the oracle's pretty-printer for the wrapped
/// "Reserved prefixes" header (measured empirically: a line breaks before the
/// next word once it would exceed column 69).
const FILL_WIDTH: usize = 69;

/// Reserved fact names (used as protocol facts -> "Reserved names").
const RESERVED_FACTS: &[&str] = &["K", "KU", "KD"];
/// Special I/O facts handled by the special-fact / Fr checks.
const SPECIAL_FACTS: &[&str] = &["In", "Out", "Fr"];

fn is_reserved(name: &str) -> bool {
    RESERVED_FACTS.contains(&name)
}
fn is_special(name: &str) -> bool {
    SPECIAL_FACTS.contains(&name)
}
fn is_builtin_factname(name: &str) -> bool {
    is_reserved(name) || is_special(name)
}

// ---------------------------------------------------------------------------
// AST traversal helpers
// ---------------------------------------------------------------------------

/// The protocol rules of a theory (excludes intruder rules).
pub fn protocol_rules(thy: &Theory) -> Vec<&Rule> {
    thy.items
        .iter()
        .filter_map(|it| match it {
            TheoryItem::Rule(r) => Some(r),
            _ => None,
        })
        .collect()
}

fn collect_term_vars(t: &Term, out: &mut Vec<VarSpec>) {
    match t {
        Term::Var(v) => out.push(v.clone()),
        Term::App(_, args) | Term::Pair(args) => {
            for a in args {
                collect_term_vars(a, out);
            }
        }
        Term::AlgApp(_, a, b) | Term::Diff(a, b) | Term::BinOp(_, a, b) => {
            collect_term_vars(a, out);
            collect_term_vars(b, out);
        }
        Term::PatMatch(inner) => collect_term_vars(inner, out),
        _ => {}
    }
}

fn collect_fact_vars(f: &Fact, out: &mut Vec<VarSpec>) {
    for a in &f.args {
        collect_term_vars(a, out);
    }
}

fn collect_facts_vars(fs: &[Fact], out: &mut Vec<VarSpec>) {
    for f in fs {
        collect_fact_vars(f, out);
    }
}

/// Collect public-constant literals (name only) from a term.
fn collect_pub_lits(t: &Term, out: &mut Vec<String>) {
    match t {
        Term::PubLit(s) => out.push(s.clone()),
        Term::App(_, args) | Term::Pair(args) => {
            for a in args {
                collect_pub_lits(a, out);
            }
        }
        Term::AlgApp(_, a, b) | Term::Diff(a, b) | Term::BinOp(_, a, b) => {
            collect_pub_lits(a, out);
            collect_pub_lits(b, out);
        }
        Term::PatMatch(inner) => collect_pub_lits(inner, out),
        _ => {}
    }
}

/// Collect fresh-name literals (`~'foo'`) from a term, rendered as the oracle
/// prints them.
fn collect_fresh_lits(t: &Term, out: &mut Vec<String>) {
    match t {
        Term::FreshLit(_) => out.push(pp_term(t)),
        Term::App(_, args) | Term::Pair(args) => {
            for a in args {
                collect_fresh_lits(a, out);
            }
        }
        Term::AlgApp(_, a, b) | Term::Diff(a, b) | Term::BinOp(_, a, b) => {
            collect_fresh_lits(a, out);
            collect_fresh_lits(b, out);
        }
        Term::PatMatch(inner) => collect_fresh_lits(inner, out),
        _ => {}
    }
}

/// Collect maximal multiplication (`*`) subterms of a term, rendered as the
/// oracle prints them. A product is maximal: its own operands are not
/// descended into.
fn collect_mult_terms(t: &Term, out: &mut Vec<String>) {
    if let Term::BinOp(BinOp::Mult, _, _) = t {
        out.push(pp_term(t));
        return;
    }
    match t {
        Term::App(_, args) | Term::Pair(args) => {
            for a in args {
                collect_mult_terms(a, out);
            }
        }
        Term::AlgApp(_, a, b) | Term::Diff(a, b) | Term::BinOp(_, a, b) => {
            collect_mult_terms(a, out);
            collect_mult_terms(b, out);
        }
        Term::PatMatch(inner) => collect_mult_terms(inner, out),
        _ => {}
    }
}

/// Indent every line of a (possibly multi-line) block by `n` spaces.
fn indent_block(s: &str, n: usize) -> String {
    let pad = " ".repeat(n);
    s.lines()
        .map(|l| format!("{}{}", pad, l))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Word-fill (Wadler/Leijen `fillSep`) at the given `width` with a leading
/// `indent` of spaces on every line. A word is pushed onto the current line
/// while it fits (column + 1 + word <= width), otherwise a new line begins.
fn fill_words(words: &[String], indent: usize, width: usize) -> String {
    let pad = " ".repeat(indent);
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut col = 0usize;
    for w in words {
        let wlen = w.chars().count();
        if cur.is_empty() {
            cur = format!("{}{}", pad, w);
            col = indent + wlen;
        } else if col + 1 + wlen <= width {
            cur.push(' ');
            cur.push_str(w);
            col += 1 + wlen;
        } else {
            lines.push(std::mem::take(&mut cur));
            cur = format!("{}{}", pad, w);
            col = indent + wlen;
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines.join("\n")
}

fn var_needs_binding(v: &VarSpec) -> bool {
    matches!(
        v.sort,
        SortHint::Fresh | SortHint::Msg | SortHint::Untagged | SortHint::Nat
    )
}

fn var_key(v: &VarSpec) -> (String, u64, i32) {
    (v.name.clone(), v.idx, sort_rank(v.sort))
}

fn sort_rank(s: SortHint) -> i32 {
    match s {
        SortHint::Msg => 0,
        SortHint::Pub => 1,
        SortHint::Fresh => 2,
        SortHint::Node => 3,
        SortHint::Nat => 4,
        SortHint::Untagged => 5,
        SortHint::Suffix(_) => 6,
    }
}

fn dedup_vars(mut vs: Vec<VarSpec>) -> Vec<VarSpec> {
    vs.sort_by_key(var_key);
    vs.dedup_by_key(|v| var_key(v));
    vs
}

// ---------------------------------------------------------------------------
// 1. Unbound variables
// ---------------------------------------------------------------------------
pub fn unbound_variables(thy: &Theory) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for r in protocol_rules(thy) {
        // Bound = variables in premises plus let-bound variables.
        let mut bound: Vec<VarSpec> = Vec::new();
        collect_facts_vars(&r.premises, &mut bound);
        for lb in &r.let_block {
            collect_term_vars(&lb.var, &mut bound);
        }
        let bound: std::collections::HashSet<(String, u64, i32)> =
            bound.iter().map(var_key).collect();

        // Used = variables in actions, conclusions, embedded restrictions,
        // and the right-hand-sides of let bindings.
        let mut used: Vec<VarSpec> = Vec::new();
        collect_facts_vars(&r.actions, &mut used);
        collect_facts_vars(&r.conclusions, &mut used);
        for lb in &r.let_block {
            collect_term_vars(&lb.value, &mut used);
        }

        let mut unbound: Vec<VarSpec> = used
            .into_iter()
            .filter(var_needs_binding)
            .filter(|v| !bound.contains(&var_key(v)))
            .collect();
        unbound = dedup_vars(unbound);
        // Report order: by base name then index.
        unbound.sort_by(|a, b| a.name.cmp(&b.name).then(a.idx.cmp(&b.idx)));

        if !unbound.is_empty() {
            let names: Vec<String> = unbound.iter().map(pp_var).collect();
            entries.push(format!(
                "  rule `{}' has unbound variables: \n    {}",
                r.name,
                names.join(", ")
            ));
        }
    }
    if entries.is_empty() {
        vec![]
    } else {
        vec![WfError::new(T_UNBOUND, entries.join("\n  \n"))]
    }
}

// ---------------------------------------------------------------------------
// 3. Variable with mismatching sorts or capitalization
// ---------------------------------------------------------------------------
pub fn mismatching_sorts(thy: &Theory) -> Vec<WfError> {
    let mut rule_entries: Vec<String> = Vec::new();
    for r in protocol_rules(thy) {
        let mut vars: Vec<VarSpec> = Vec::new();
        collect_facts_vars(&r.premises, &mut vars);
        collect_facts_vars(&r.actions, &mut vars);
        collect_facts_vars(&r.conclusions, &mut vars);

        // Group by lowercased base name; a group with >1 distinct variant
        // (prefix+name, ignoring index) is a conflict.
        let mut groups: Vec<(String, Vec<String>)> = Vec::new(); // (lc key, variants)
        for v in &vars {
            let key = v.name.to_lowercase();
            let variant = variant_repr(v);
            match groups.iter_mut().find(|(k, _)| *k == key) {
                Some((_, variants)) => {
                    if !variants.contains(&variant) {
                        variants.push(variant);
                    }
                }
                None => groups.push((key, vec![variant])),
            }
        }
        let mut conflicts: Vec<Vec<String>> = groups
            .into_iter()
            .filter(|(_, vs)| vs.len() > 1)
            .map(|(_, mut vs)| {
                vs.sort();
                vs
            })
            .collect();
        if conflicts.is_empty() {
            continue;
        }
        // deterministic ordering of groups within a rule
        conflicts.sort();
        let mut body = format!("  rule `{}': ", r.name);
        for (i, variants) in conflicts.iter().enumerate() {
            body.push_str(&format!("\n    {}. {}", i + 1, variants.join(", ")));
        }
        rule_entries.push(body);
    }
    if rule_entries.is_empty() {
        return vec![];
    }
    let header = "Possible reasons:\n1. Identifiers are case sensitive, i.e.,'x' and 'X' are considered to be different.\n2. The same holds for sorts:, i.e., '$x', 'x', and '~x' are considered to be different.\n";
    let msg = format!("{}\n{}", header, rule_entries.join("\n  \n"));
    vec![WfError::new(T_SORTS, msg)]
}

/// A variable's sort+name representation without the numeric index.
fn variant_repr(v: &VarSpec) -> String {
    let prefix = match v.sort {
        SortHint::Fresh => "~",
        SortHint::Pub => "$",
        SortHint::Nat => "%",
        SortHint::Node => "#",
        _ => "",
    };
    format!("{}{}", prefix, v.name)
}

// ---------------------------------------------------------------------------
// 4. Reserved names
// ---------------------------------------------------------------------------
pub fn reserved_names(thy: &Theory) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for r in protocol_rules(thy) {
        // On the left/right the reserved set is {K,KU,KD}; in the middle
        // (actions) the I/O facts In/Out/Fr are reserved too (observed z9/z11).
        for (facts, phrase, middle) in [
            (&r.premises, "left-hand-side", false),
            (&r.actions, "the middle", true),
            (&r.conclusions, "the right-hand-side", false),
        ] {
            let hits: Vec<&Fact> = facts
                .iter()
                .filter(|f| is_reserved(&f.name) || (middle && is_special(&f.name)))
                .collect();
            if hits.is_empty() {
                continue;
            }
            let rendered: Vec<String> = hits.iter().map(|f| pp_fact(f)).collect();
            entries.push(format!(
                "  Rule `{}' contains facts with reserved names on {}:\n    {}",
                r.name,
                phrase,
                rendered.join(", ")
            ));
        }
    }
    if entries.is_empty() {
        vec![]
    } else {
        vec![WfError::new(T_RESERVED, entries.join("\n  \n"))]
    }
}

// ---------------------------------------------------------------------------
// 5. Fr facts must only use a fresh- or a msg-variable
// ---------------------------------------------------------------------------
pub fn fr_facts(thy: &Theory) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for r in protocol_rules(thy) {
        for facts in [&r.premises, &r.conclusions] {
            for f in facts {
                if f.name != "Fr" {
                    continue;
                }
                let ok = f.args.len() == 1
                    && matches!(
                        &f.args[0],
                        Term::Var(v) if matches!(v.sort, SortHint::Fresh | SortHint::Msg | SortHint::Untagged)
                    );
                if !ok {
                    entries.push(format!("  rule `{}' fact: {}", r.name, pp_fact(f)));
                }
            }
        }
    }
    if entries.is_empty() {
        vec![]
    } else {
        vec![WfError::new(T_FR, entries.join("\n  \n"))]
    }
}

// ---------------------------------------------------------------------------
// 6. Special facts (disallowed I/O facts in wrong position)
// ---------------------------------------------------------------------------
pub fn special_facts(thy: &Theory) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for r in protocol_rules(thy) {
        // Premise side: `Out` is disallowed.
        let lhs: Vec<String> = r
            .premises
            .iter()
            .filter(|f| f.name == "Out")
            .map(|f| pp_fact(f))
            .collect();
        if !lhs.is_empty() {
            entries.push(format!(
                "  rule `{}' uses disallowed facts on left-hand-side:\n    {}",
                r.name,
                lhs.join(", ")
            ));
        }
        // Conclusion side: `In` and `Fr` are disallowed.
        let rhs: Vec<String> = r
            .conclusions
            .iter()
            .filter(|f| f.name == "In" || f.name == "Fr")
            .map(|f| pp_fact(f))
            .collect();
        if !rhs.is_empty() {
            entries.push(format!(
                "  rule `{}' uses disallowed facts on right-hand-side:\n    {}",
                r.name,
                rhs.join(", ")
            ));
        }
    }
    if entries.is_empty() {
        vec![]
    } else {
        vec![WfError::new(T_SPECIAL, entries.join("\n  \n"))]
    }
}

// ---------------------------------------------------------------------------
// 7 & 8. Fact arity / multiplicity issues
// ---------------------------------------------------------------------------

struct FactUse {
    label: &'static str, // "Rule" or "Lemma"
    owner: String,       // rule/lemma name
    arity: usize,
    persistent: bool,
    render: String, // pp_fact for rules; raw Haskell `Fact {..}` show for lemmas
}

/// Raw Haskell `Show` of a lemma-sourced fact, as the oracle prints it in the
/// arity/multiplicity blocks (observed r3_lemarity / r3_lemmult):
///   Fact {factTag = ProtoFact Linear "Act" 2, factAnnotations = fromList [],
///         factTerms = [Bound 2,Bound 1]}
/// `factTerms` uses the same de Bruijn / Free rendering as the Formula-terms
/// check (the binder `stack` is the quantifier context at the atom).
fn show_haskell_fact(f: &Fact, stack: &[String]) -> String {
    let mult = if f.persistent { "Persistent" } else { "Linear" };
    let terms: Vec<String> = f.args.iter().map(|a| show_wf_term(a, stack)).collect();
    format!(
        "Fact {{factTag = ProtoFact {} \"{}\" {}, factAnnotations = fromList [], factTerms = [{}]}}",
        mult,
        f.name,
        f.args.len(),
        terms.join(",")
    )
}

/// Collect action-fact uses from a lemma formula (label "Lemma"), tracking the
/// quantifier binder stack for the de Bruijn rendering of the fact terms.
fn gather_formula_facts(
    f: &Formula,
    stack: &mut Vec<String>,
    owner: &str,
    out: &mut Vec<(String, FactUse)>,
) {
    match f {
        Formula::False | Formula::True => {}
        Formula::Atom(Atom::Action(fact, _)) => {
            out.push((
                fact.name.clone(),
                FactUse {
                    label: "Lemma",
                    owner: owner.to_string(),
                    arity: fact.args.len(),
                    persistent: fact.persistent,
                    render: show_haskell_fact(fact, stack),
                },
            ));
        }
        Formula::Atom(_) => {}
        Formula::Not(g) => gather_formula_facts(g, stack, owner, out),
        Formula::And(a, b)
        | Formula::Or(a, b)
        | Formula::Implies(a, b)
        | Formula::Iff(a, b) => {
            gather_formula_facts(a, stack, owner, out);
            gather_formula_facts(b, stack, owner, out);
        }
        Formula::Forall(vs, g) | Formula::Exists(vs, g) => {
            let n = vs.len();
            for v in vs {
                stack.push(v.name.clone());
            }
            gather_formula_facts(g, stack, owner, out);
            for _ in 0..n {
                stack.pop();
            }
        }
    }
}

/// Gather every fact occurrence in the theory (name -> uses) in source order.
/// Protocol rules contribute their premise/action/conclusion facts (rendered by
/// the term pretty-printer); lemmas contribute their formula action facts
/// (rendered as the raw Haskell `Fact {..}` show). Restrictions do NOT
/// contribute (observed r3_restrarity: a restriction action arity mismatch is
/// silent).
fn gather_fact_uses(thy: &Theory) -> Vec<(String, Vec<FactUse>)> {
    let mut map: Vec<(String, Vec<FactUse>)> = Vec::new();
    let push = |name: String, u: FactUse, map: &mut Vec<(String, Vec<FactUse>)>| {
        match map.iter_mut().find(|(n, _)| *n == name) {
            Some((_, uses)) => uses.push(u),
            None => map.push((name, vec![u])),
        }
    };
    for it in &thy.items {
        match it {
            TheoryItem::Rule(r) => {
                for facts in [&r.premises, &r.actions, &r.conclusions] {
                    for f in facts {
                        push(
                            f.name.clone(),
                            FactUse {
                                label: "Rule",
                                owner: r.name.clone(),
                                arity: f.args.len(),
                                persistent: f.persistent,
                                render: pp_fact(f),
                            },
                            &mut map,
                        );
                    }
                }
            }
            TheoryItem::Lemma(l) => {
                let mut stack: Vec<String> = Vec::new();
                let mut uses: Vec<(String, FactUse)> = Vec::new();
                gather_formula_facts(&l.formula, &mut stack, &l.name, &mut uses);
                for (name, u) in uses {
                    push(name, u, &mut map);
                }
            }
            _ => {}
        }
    }
    map
}

fn render_fact_blocks(conflicts: &[(String, Vec<String>)], intro1: &str, intro2: &str) -> String {
    // conflicts: (lowercased fact name, item lines) already ordered.
    let mut body = format!("{}\n{}\n  ", intro1, intro2);
    for (i, (lname, items)) in conflicts.iter().enumerate() {
        let block = format!("  Fact `{}':\n\n{}\n  ", lname, items.join("\n    \n"));
        if i == 0 {
            body.push_str("\n\n");
        } else {
            body.push('\n');
        }
        body.push_str(&block);
    }
    body
}

// ---------------------------------------------------------------------------
// Fact capitalization issues (precedes Fact arity issues)
// ---------------------------------------------------------------------------
// Fact names that are equal under ASCII-lowercasing but differ in their exact
// spelling (e.g. `Send` vs `SEND`) are distinct facts. Reported like the arity
// block, but EVERY occurrence is listed (no per-(rule,cap) deduplication -
// observed r3_capord: `Send` twice in one rule yields two items).
pub fn fact_capitalization(thy: &Theory) -> Vec<WfError> {
    struct Occ {
        name: String,
        rule: String,
        pp: String,
    }
    let mut occ: Vec<Occ> = Vec::new();
    for r in protocol_rules(thy) {
        for facts in [&r.premises, &r.actions, &r.conclusions] {
            for f in facts {
                occ.push(Occ {
                    name: f.name.clone(),
                    rule: r.name.clone(),
                    pp: pp_fact(f),
                });
            }
        }
    }
    // Group by lowercased name, preserving first-seen order.
    let mut groups: Vec<(String, Vec<&Occ>)> = Vec::new();
    for o in &occ {
        let key = o.name.to_lowercase();
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, v)) => v.push(o),
            None => groups.push((key, vec![o])),
        }
    }
    let mut conflicts: Vec<(String, Vec<String>)> = Vec::new();
    for (key, os) in &groups {
        let distinct: std::collections::BTreeSet<&String> = os.iter().map(|o| &o.name).collect();
        if distinct.len() < 2 {
            continue; // no capitalization conflict
        }
        let items: Vec<String> = os
            .iter()
            .enumerate()
            .map(|(i, o)| {
                format!(
                    "    {}. Rule `{}', capitalization \"{}\"\n         {}",
                    i + 1,
                    o.rule,
                    o.name,
                    o.pp
                )
            })
            .collect();
        conflicts.push((key.clone(), items));
    }
    if conflicts.is_empty() {
        return vec![];
    }
    conflicts.sort_by(|a, b| a.0.cmp(&b.0));
    let msg = render_fact_blocks(
        &conflicts,
        "Fact names are case-sensitive, different capitalizations are considered as different facts, i.e., Fact() is different from FAct(). ",
        "Check the capitalization of your fact names.",
    );
    vec![WfError::new(T_FACT_CAP, msg)]
}

pub fn fact_arity(thy: &Theory) -> Vec<WfError> {
    let uses = gather_fact_uses(thy);
    let mut conflicts: Vec<(String, Vec<String>)> = Vec::new();
    for (name, us) in &uses {
        let arities: std::collections::BTreeSet<usize> = us.iter().map(|u| u.arity).collect();
        if arities.len() < 2 {
            continue;
        }
        // one item per distinct (label, owner, arity), first render kept.
        let mut seen: Vec<(&str, String, usize)> = Vec::new();
        let mut items: Vec<String> = Vec::new();
        for u in us {
            let k = (u.label, u.owner.clone(), u.arity);
            if seen.contains(&k) {
                continue;
            }
            seen.push(k);
            let n = items.len() + 1;
            items.push(format!(
                "    {}. {} `{}', arity {}\n         {}",
                n, u.label, u.owner, u.arity, u.render
            ));
        }
        conflicts.push((name.to_lowercase(), items));
    }
    if conflicts.is_empty() {
        return vec![];
    }
    conflicts.sort_by(|a, b| a.0.cmp(&b.0));
    let msg = render_fact_blocks(
        &conflicts,
        "Same fact is used with different arities, i.e., Fact('A','B') is different from Fact('A'). ",
        "Check the arguments of your facts.",
    );
    vec![WfError::new(T_ARITY, msg)]
}

pub fn fact_multiplicity(thy: &Theory) -> Vec<WfError> {
    let uses = gather_fact_uses(thy);
    let mut conflicts: Vec<(String, Vec<String>)> = Vec::new();
    for (name, us) in &uses {
        let mults: std::collections::BTreeSet<bool> = us.iter().map(|u| u.persistent).collect();
        if mults.len() < 2 {
            continue;
        }
        let mut seen: Vec<(&str, String, bool)> = Vec::new();
        let mut items: Vec<String> = Vec::new();
        for u in us {
            let k = (u.label, u.owner.clone(), u.persistent);
            if seen.contains(&k) {
                continue;
            }
            seen.push(k);
            let n = items.len() + 1;
            let m = if u.persistent { "Persistent" } else { "Linear" };
            items.push(format!(
                "    {}. {} `{}', multiplicity (persistence) {}\n         {}",
                n, u.label, u.owner, m, u.render
            ));
        }
        conflicts.push((name.to_lowercase(), items));
    }
    if conflicts.is_empty() {
        return vec![];
    }
    conflicts.sort_by(|a, b| a.0.cmp(&b.0));
    let msg = render_fact_blocks(
        &conflicts,
        "Same fact is used with different multiplicities, i.e., !Fact() (Persistent fact) exists along with Fact() (Linear) in your rules. ",
        "Check the multiplicity (persistence) of your facts.",
    );
    vec![WfError::new(T_MULT, msg)]
}

// ---------------------------------------------------------------------------
// 9. Facts occur in the left-hand-side but not in any right-hand-side
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct FactId {
    name: String,
    arity: usize,
    persistent: bool,
    rule: String,
}

impl FactId {
    fn ident(&self) -> (String, usize, bool) {
        (self.name.clone(), self.arity, self.persistent)
    }
    fn render(&self) -> String {
        format!(
            "in rule \"{}\":  factName `{}' arity: {} multiplicity: {}",
            self.rule,
            self.name,
            self.arity,
            if self.persistent { "Persistent" } else { "Linear" }
        )
    }
}

pub fn fact_lhs_occur_no_rhs(thy: &Theory) -> Vec<WfError> {
    let mut lhs: Vec<FactId> = Vec::new();
    let mut rhs: Vec<FactId> = Vec::new();
    for r in protocol_rules(thy) {
        for f in &r.premises {
            if is_builtin_factname(&f.name) {
                continue;
            }
            lhs.push(FactId {
                name: f.name.clone(),
                arity: f.args.len(),
                persistent: f.persistent,
                rule: r.name.clone(),
            });
        }
        for f in &r.conclusions {
            if is_builtin_factname(&f.name) {
                continue;
            }
            rhs.push(FactId {
                name: f.name.clone(),
                arity: f.args.len(),
                persistent: f.persistent,
                rule: r.name.clone(),
            });
        }
    }
    let rhs_idents: std::collections::HashSet<(String, usize, bool)> =
        rhs.iter().map(|f| f.ident()).collect();

    // LHS-only identities, first occurrence, in source order.
    let mut seen: Vec<(String, usize, bool)> = Vec::new();
    let mut items: Vec<String> = Vec::new();
    for f in &lhs {
        if rhs_idents.contains(&f.ident()) || seen.contains(&f.ident()) {
            continue;
        }
        seen.push(f.ident());
        let n = items.len() + 1;
        let mut line = format!("  {}. {}", n, f.render());
        if let Some(sugg) = nearest_rhs(&f.name, &rhs) {
            line.push_str(&format!(". Perhaps you want to use the fact {}", sugg.render()));
        }
        items.push(line);
    }
    if items.is_empty() {
        vec![]
    } else {
        vec![WfError::new(T_LHSRHS, items.join("\n  \n"))]
    }
}

/// Nearest RHS fact by name edit distance, if within threshold (<= 3).
fn nearest_rhs<'a>(name: &str, rhs: &'a [FactId]) -> Option<&'a FactId> {
    let mut best: Option<(usize, &FactId)> = None;
    for f in rhs {
        let d = edit_distance(name, &f.name);
        match best {
            Some((bd, _)) if d >= bd => {}
            _ => best = Some((d, f)),
        }
    }
    match best {
        Some((d, f)) if d <= 3 => Some(f),
        _ => None,
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

// ---------------------------------------------------------------------------
// 2. Public constants with mismatching capitalization (public-names report)
// ---------------------------------------------------------------------------

pub fn public_names_report(thy: &Theory) -> Vec<WfError> {
    let mut pairs: Vec<(String, String)> = Vec::new(); // (constName, ruleName)
    for r in protocol_rules(thy) {
        let mut names: Vec<String> = Vec::new();
        for facts in [&r.premises, &r.actions, &r.conclusions] {
            for f in facts {
                for a in &f.args {
                    collect_pub_lits(a, &mut names);
                }
            }
        }
        for n in names {
            let pair = (n, r.name.clone());
            if !pairs.contains(&pair) {
                pairs.push(pair);
            }
        }
    }
    public_names_report_from_pairs(pairs)
}

pub fn public_names_report_from_pairs(pairs: Vec<(String, String)>) -> Vec<WfError> {
    // Group by lowercased constant name, preserving first-seen order.
    let mut groups: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for (name, rule) in &pairs {
        let key = name.to_lowercase();
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, v)) => v.push((name.clone(), rule.clone())),
            None => groups.push((key, vec![(name.clone(), rule.clone())])),
        }
    }
    let mut items: Vec<String> = Vec::new();
    for (_, occ) in &groups {
        let distinct: std::collections::BTreeSet<&String> = occ.iter().map(|(n, _)| n).collect();
        if distinct.len() < 2 {
            continue; // no capitalization conflict
        }
        // Group occurrences by rule, preserving first-seen rule order.
        let mut by_rule: Vec<(String, Vec<String>)> = Vec::new();
        for (n, rule) in occ {
            match by_rule.iter_mut().find(|(rr, _)| rr == rule) {
                Some((_, ns)) => {
                    if !ns.contains(n) {
                        ns.push(n.clone());
                    }
                }
                None => by_rule.push((rule.clone(), vec![n.clone()])),
            }
        }
        let locs: Vec<String> = by_rule
            .into_iter()
            .map(|(rule, mut ns)| {
                ns.sort();
                let quoted: Vec<String> = ns.iter().map(|n| format!("'{}'", n)).collect();
                format!("rule \"{}\":  name {}", rule, quoted.join(", "))
            })
            .collect();
        let n = items.len() + 1;
        items.push(format!("  {}. {}", n, locs.join(", ")));
    }
    if items.is_empty() {
        return vec![];
    }
    let header = "Identifiers are case-sensitive, mismatched capitalizations are considered as different, i.e., 'ID' is different from 'id'. Check the capitalization of your identifiers.";
    let msg = format!("{}\n\n{}", header, items.join("\n  \n"));
    vec![WfError::new(T_PUBNAMES, msg)]
}

// ---------------------------------------------------------------------------
// 10. Formula terms (ill-formed terms in lemma / restriction formulas)
// ---------------------------------------------------------------------------
// A lemma/restriction formula may only use terms built from public constants
// and BOUND node/message variables via non-reducible function symbols. A term
// is reported "of the wrong form" if it contains a FREE variable or a REDUCIBLE
// function symbol. The whole offending top-level term (each argument of each
// atom) is reported, rendered in the oracle's raw term representation:
//   - a bound variable  -> `Bound N` (de Bruijn: 0 = innermost binder)
//   - a free  variable  -> `Free <pp_var>`  (keeps the sort prefix)
//   - a function app    -> `f(a,b)`  (args comma-joined, NO space)
//   - a tuple           -> `pair(a,pair(b,c))` (right-nested binary pairs)
//   - a public constant -> `'name'`
// The set of reducible function symbol names is caller-supplied (the caller
// computes reducibility from the equation theory / Maude); this module only
// consumes it. `formula_terms` is the zero-reducible-symbols convenience
// wrapper (free variables only). See BEHAVIOR.md.

const FORMULA_TERMS_HELP: &str = "  The only allowed terms are public constants and bound node and\n  message variables. If you encounter free message variables, then\n  you might have forgotten a #-prefix. Sort prefixes can only be\n  dropped where this is unambiguous. Moreover, reducible function\n  symbols are disallowed.";

/// De Bruijn index of `name` on the binder stack (outermost pushed first): the
/// innermost matching binder is 0. `None` if the name is not bound (free).
/// Binders are matched by NAME only (see BEHAVIOR.md round-2 fix).
fn debruijn_index(stack: &[String], name: &str) -> Option<usize> {
    stack
        .iter()
        .rposition(|b| b == name)
        .map(|pos| stack.len() - 1 - pos)
}

/// Render a term in the oracle's raw "wrong form" representation.
fn show_wf_term(t: &Term, stack: &[String]) -> String {
    match t {
        Term::Var(v) => match debruijn_index(stack, &v.name) {
            Some(idx) => format!("Bound {}", idx),
            None => format!("Free {}", pp_var(v)),
        },
        Term::PubLit(s) => format!("'{}'", s),
        Term::FreshLit(s) => format!("~'{}'", s),
        Term::NatLit(s) => format!("%'{}'", s),
        Term::Number(n) => n.to_string(),
        Term::NumberOne => "1".to_string(),
        Term::NatOne => "%1".to_string(),
        Term::DhNeutral => "DH_neutral".to_string(),
        Term::App(name, args) => {
            if args.is_empty() {
                name.clone()
            } else {
                let parts: Vec<String> = args.iter().map(|a| show_wf_term(a, stack)).collect();
                format!("{}({})", name, parts.join(","))
            }
        }
        Term::AlgApp(name, a, b) => {
            format!("{}({},{})", name, show_wf_term(a, stack), show_wf_term(b, stack))
        }
        Term::Pair(items) => show_wf_pair(items, stack),
        Term::Diff(a, b) => {
            format!("diff({},{})", show_wf_term(a, stack), show_wf_term(b, stack))
        }
        Term::BinOp(op, a, b) => {
            // Raw operator function name (best-effort; AC/DH operators in
            // formula terms are rare - see BEHAVIOR.md gaps).
            let name = match op {
                BinOp::Exp => "exp",
                BinOp::Mult => "mult",
                BinOp::Union => "union",
                BinOp::Xor => "xor",
                BinOp::NatPlus => "tadd",
            };
            format!("{}({},{})", name, show_wf_term(a, stack), show_wf_term(b, stack))
        }
        Term::PatMatch(inner) => show_wf_term(inner, stack),
    }
}

/// Render a tuple as right-nested binary `pair(...)` applications.
fn show_wf_pair(items: &[Term], stack: &[String]) -> String {
    match items {
        [] => "pair".to_string(),
        [only] => show_wf_term(only, stack),
        [head, rest @ ..] => {
            format!("pair({},{})", show_wf_term(head, stack), show_wf_pair(rest, stack))
        }
    }
}

/// True iff `t` contains a free variable or a reducible function symbol, i.e.
/// the term is "of the wrong form" for a trace formula.
fn term_is_ill_formed(t: &Term, stack: &[String], reducible: &std::collections::BTreeSet<String>) -> bool {
    match t {
        Term::Var(v) => debruijn_index(stack, &v.name).is_none(), // free -> ill
        Term::PubLit(_)
        | Term::FreshLit(_)
        | Term::NatLit(_)
        | Term::Number(_)
        | Term::NumberOne
        | Term::NatOne
        | Term::DhNeutral => false,
        Term::App(name, args) => {
            reducible.contains(name) || args.iter().any(|a| term_is_ill_formed(a, stack, reducible))
        }
        Term::AlgApp(name, a, b) => {
            reducible.contains(name)
                || term_is_ill_formed(a, stack, reducible)
                || term_is_ill_formed(b, stack, reducible)
        }
        Term::Pair(items) => items.iter().any(|a| term_is_ill_formed(a, stack, reducible)),
        Term::Diff(a, b) | Term::BinOp(_, a, b) => {
            term_is_ill_formed(a, stack, reducible) || term_is_ill_formed(b, stack, reducible)
        }
        Term::PatMatch(inner) => term_is_ill_formed(inner, stack, reducible),
    }
}

/// The argument terms of an atom, in the oracle's reporting order. For an
/// action atom the TEMPORAL variable is reported before the fact arguments
/// (observed via probe r3_actord).
fn atom_terms(a: &Atom) -> Vec<&Term> {
    match a {
        Atom::Eq(x, y) | Atom::Less(x, y) | Atom::LessMset(x, y) | Atom::Subterm(x, y) => {
            vec![x, y]
        }
        Atom::Action(f, t) => {
            let mut v = vec![t];
            v.extend(f.args.iter());
            v
        }
        Atom::Last(t) => vec![t],
        Atom::Pred(f) => f.args.iter().collect(),
    }
}

/// Walk a formula in source order collecting the RENDERED strings of every
/// ill-formed atom term. Not deduplicated (probe r3_shapes: `x = y & x = y`
/// reports `Free y` twice).
fn collect_ill_terms(
    f: &Formula,
    stack: &mut Vec<String>,
    reducible: &std::collections::BTreeSet<String>,
    out: &mut Vec<String>,
) {
    match f {
        Formula::False | Formula::True => {}
        Formula::Atom(a) => {
            for t in atom_terms(a) {
                if term_is_ill_formed(t, stack, reducible) {
                    out.push(show_wf_term(t, stack));
                }
            }
        }
        Formula::Not(g) => collect_ill_terms(g, stack, reducible, out),
        Formula::And(a, b)
        | Formula::Or(a, b)
        | Formula::Implies(a, b)
        | Formula::Iff(a, b) => {
            collect_ill_terms(a, stack, reducible, out);
            collect_ill_terms(b, stack, reducible, out);
        }
        Formula::Forall(vs, g) | Formula::Exists(vs, g) => {
            let n = vs.len();
            for v in vs {
                stack.push(v.name.clone());
            }
            collect_ill_terms(g, stack, reducible, out);
            for _ in 0..n {
                stack.pop();
            }
        }
    }
}

/// Lay out `tokens` after `prefix` with a fillSep (paragraph fill): place a
/// token on the current line while `col + 1 + width <= width`, else break to a
/// new line indented `indent`. Each token but the last carries a trailing
/// comma. Matches the oracle's wrapping at column 69 (probes r3_wrap/r3_w2).
fn fill_after_prefix(prefix: &str, tokens: &[String], indent: usize, width: usize) -> String {
    let mut line = prefix.to_string();
    let mut col = prefix.chars().count();
    let pad = " ".repeat(indent);
    let n = tokens.len();
    for (i, tok) in tokens.iter().enumerate() {
        let piece = if i + 1 < n {
            format!("{},", tok)
        } else {
            tok.clone()
        };
        let w = piece.chars().count();
        if col + 1 + w <= width {
            line.push(' ');
            line.push_str(&piece);
            col += 1 + w;
        } else {
            line.push('\n');
            line.push_str(&pad);
            line.push_str(&piece);
            col = indent + w;
        }
    }
    line
}

fn formula_terms_entry(entity: &str, name: &str, terms: &[String]) -> String {
    let prefix = format!("  {} `{}' uses terms of the wrong form:", entity, name);
    let tokens: Vec<String> = terms.iter().map(|t| format!("`{}'", t)).collect();
    let head = fill_after_prefix(&prefix, &tokens, 4, FILL_WIDTH);
    format!("{}\n  \n{}", head, FORMULA_TERMS_HELP)
}

/// Full "Formula terms" check: reports ill-formed terms (free variables and
/// applications of any symbol in `reducible`) in lemma/restriction formulas.
pub fn formula_terms_reducible(
    thy: &Theory,
    reducible: &std::collections::BTreeSet<String>,
) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for it in &thy.items {
        let (entity, name, formula) = match it {
            TheoryItem::Lemma(l) => ("Lemma", &l.name, &l.formula),
            TheoryItem::Restriction(r) | TheoryItem::LegacyAxiom(r) => {
                ("Restriction", &r.name, &r.formula)
            }
            _ => continue,
        };
        let mut stack: Vec<String> = Vec::new();
        let mut terms: Vec<String> = Vec::new();
        collect_ill_terms(formula, &mut stack, reducible, &mut terms);
        if !terms.is_empty() {
            entries.push(formula_terms_entry(entity, name, &terms));
        }
    }
    if entries.is_empty() {
        vec![]
    } else {
        vec![WfError::new(T_FORMULA_TERMS, entries.join("\n  \n"))]
    }
}

/// Convenience wrapper: the free-variable-only "Formula terms" check (no
/// reducible symbols). `check_theory` uses this; the reducible-aware entry
/// point is `formula_terms_reducible`.
pub fn formula_terms(thy: &Theory) -> Vec<WfError> {
    formula_terms_reducible(thy, &std::collections::BTreeSet::new())
}

// ---------------------------------------------------------------------------
// 11. Formula guardedness (best-effort; single-line formula printer)
// ---------------------------------------------------------------------------

pub fn formula_guardedness(thy: &Theory) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for it in &thy.items {
        let (name, formula) = match it {
            TheoryItem::Lemma(l) => (&l.name, &l.formula),
            _ => continue,
        };
        if let Some((fail, sub)) = find_guard_failure(formula) {
            let reason = match fail {
                GuardFail::Unguarded(vars) => {
                    let vs: Vec<String> =
                        vars.iter().map(|v| format!("'{}'", pp_var(v))).collect();
                    format!(
                        "unguarded variable(s) {} in the subformula",
                        vs.join(", ")
                    )
                }
                GuardFail::NoImplication => {
                    "universal quantifier without toplevel implication".to_string()
                }
            };
            // The formula is embedded as `      "..."`: the quote sits at column
            // 6, so the formula starts at column 7 with base indent 6.
            let pp_sub = crate::formula::pp_formula_wrapped(&sub, 7, 6);
            let pp_whole = crate::formula::pp_formula_wrapped(formula, 7, 6);
            entries.push(format!(
                "  Lemma `{}' cannot be converted to a guarded formula:\n    {}\n      \"{}\"\n    in the formula\n      \"{}\"",
                name, reason, pp_sub, pp_whole
            ));
        }
    }
    if entries.is_empty() {
        vec![]
    } else {
        vec![WfError::new(T_GUARD, entries.join("\n"))]
    }
}

/// Variables appearing in any Action or Pred atom anywhere in `f` (the guard
/// positions). Used as a permissive over-approximation of "guarded".
fn guard_vars(f: &Formula, out: &mut Vec<VarSpec>) {
    match f {
        Formula::Atom(Atom::Action(fact, t)) => {
            collect_fact_vars(fact, out);
            collect_term_vars(t, out);
        }
        Formula::Atom(Atom::Pred(fact)) => collect_fact_vars(fact, out),
        Formula::Atom(_) | Formula::True | Formula::False => {}
        Formula::Not(g) => guard_vars(g, out),
        Formula::And(a, b)
        | Formula::Or(a, b)
        | Formula::Implies(a, b)
        | Formula::Iff(a, b) => {
            guard_vars(a, out);
            guard_vars(b, out);
        }
        Formula::Forall(_, g) | Formula::Exists(_, g) => guard_vars(g, out),
    }
}

/// A guardedness failure and its reason.
enum GuardFail {
    /// A quantifier binds variables not guarded by an action fact.
    Unguarded(Vec<VarSpec>),
    /// A universal quantifier's body is not a top-level implication.
    NoImplication,
}

/// Guard variable names of a (guard) formula: the variables appearing in its
/// action/predicate atoms. Matched by NAME (see free_vars_formula rationale).
fn guard_var_names(f: &Formula) -> std::collections::HashSet<String> {
    let mut gv = Vec::new();
    guard_vars(f, &mut gv);
    gv.iter().map(|v| v.name.clone()).collect()
}

/// Return the first guardedness failure and the failing quantifier subformula.
///
/// A universal quantifier is guarded only when its body is a top-level
/// implication `guard ==> rest` whose antecedent's action facts bind every
/// quantified variable (observed r3_gc: a conjunction/disjunction/negation body,
/// or a bare atom body, all fail as "without toplevel implication"; the
/// antecedent alone - not the consequent - guards the variables). Existential
/// quantifiers instead take a conjunctive guard (observed: `Ex x #i. A(x) @ #i`
/// is fine); an existential whose variables are not all guarded is "unguarded".
fn find_guard_failure(f: &Formula) -> Option<(GuardFail, Formula)> {
    match f {
        Formula::Forall(vs, body) => match &**body {
            Formula::Implies(guard, rest) => {
                let gset = guard_var_names(guard);
                let unguarded: Vec<VarSpec> = vs
                    .iter()
                    .filter(|v| !gset.contains(&v.name))
                    .cloned()
                    .collect();
                if !unguarded.is_empty() {
                    return Some((GuardFail::Unguarded(unguarded), f.clone()));
                }
                find_guard_failure(guard).or_else(|| find_guard_failure(rest))
            }
            _ => Some((GuardFail::NoImplication, f.clone())),
        },
        Formula::Exists(vs, body) => {
            let gset = guard_var_names(body);
            let unguarded: Vec<VarSpec> = vs
                .iter()
                .filter(|v| !gset.contains(&v.name))
                .cloned()
                .collect();
            if !unguarded.is_empty() {
                return Some((GuardFail::Unguarded(unguarded), f.clone()));
            }
            find_guard_failure(body)
        }
        Formula::Not(g) => find_guard_failure(g),
        Formula::And(a, b)
        | Formula::Or(a, b)
        | Formula::Implies(a, b)
        | Formula::Iff(a, b) => find_guard_failure(a).or_else(|| find_guard_failure(b)),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// 12. Nat Sorts
// ---------------------------------------------------------------------------

pub fn nat_sorts(thy: &Theory) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for r in protocol_rules(thy) {
        for facts in [&r.premises, &r.actions, &r.conclusions] {
            for f in facts {
                for a in &f.args {
                    collect_nat_issues(a, &mut entries);
                }
            }
        }
    }
    if entries.is_empty() {
        vec![]
    } else {
        vec![WfError::new(T_NAT, entries.join("\n  \n"))]
    }
}

fn collect_nat_issues(t: &Term, out: &mut Vec<String>) {
    if let Term::BinOp(BinOp::NatPlus, _, _) = t {
        let term_pp = pp_term(t);
        let mut vs = Vec::new();
        collect_term_vars(t, &mut vs);
        for v in vs {
            if v.sort != SortHint::Nat {
                let entry = format!("  {} in term {} must be of sort nat", pp_var(&v), term_pp);
                if !out.contains(&entry) {
                    out.push(entry);
                }
            }
        }
    }
    // recurse
    match t {
        Term::App(_, args) | Term::Pair(args) => {
            for a in args {
                collect_nat_issues(a, out);
            }
        }
        Term::AlgApp(_, a, b) | Term::Diff(a, b) | Term::BinOp(_, a, b) => {
            collect_nat_issues(a, out);
            collect_nat_issues(b, out);
        }
        Term::PatMatch(inner) => collect_nat_issues(inner, out),
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// 13. Subterm Convergence Warning
// ---------------------------------------------------------------------------

pub fn subterm_convergence(thy: &Theory) -> Vec<WfError> {
    let mut bad: Vec<String> = Vec::new();
    for it in &thy.items {
        if let TheoryItem::Equations { convergent, eqs } = it {
            if *convergent {
                continue; // user asserted convergence
            }
            for eq in eqs {
                if !is_subterm(&eq.rhs, &eq.lhs) {
                    bad.push(format!("    {} = {}", pp_term(&eq.lhs), pp_term(&eq.rhs)));
                }
            }
        }
    }
    if bad.is_empty() {
        return vec![];
    }
    let intro = "  User-defined equations must be convergent and have the finite variant property. The following equations are not subterm convergent. If you are sure that the set of equations is nevertheless convergent and has the finite variant property, you can ignore this warning and continue ";
    let manual = " For more information, please refer to the manual : https://tamarin-prover.com/manual/master/book/010_modeling-issues.html ";
    let msg = format!("{}\n\n{}\n   \n{}", intro, bad.join("\n"), manual);
    vec![WfError::new(T_SUBTERM, msg)]
}

/// Structural subterm test: does `small` occur as a subterm of `big`?
fn is_subterm(small: &Term, big: &Term) -> bool {
    if small == big {
        return true;
    }
    match big {
        Term::App(_, args) | Term::Pair(args) => args.iter().any(|a| is_subterm(small, a)),
        Term::AlgApp(_, a, b) | Term::Diff(a, b) | Term::BinOp(_, a, b) => {
            is_subterm(small, a) || is_subterm(small, b)
        }
        Term::PatMatch(inner) => is_subterm(small, inner),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// 2. Fresh public constants
// ---------------------------------------------------------------------------
// A fresh-name literal (`~'foo'`) used directly in a rule is rejected; fresh
// names must come from `Fr(~x)` premises. Constants are collected in the order
// premises, conclusions, actions (observed via probe fpc_positions).
pub fn fresh_public_constants(thy: &Theory) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for r in protocol_rules(thy) {
        let mut lits: Vec<String> = Vec::new();
        for facts in [&r.premises, &r.conclusions, &r.actions] {
            for f in facts {
                for a in &f.args {
                    collect_fresh_lits(a, &mut lits);
                }
            }
        }
        if lits.is_empty() {
            continue;
        }
        // The constant list is a fillSep wrapped at column 69 with a 4-space
        // continuation indent (probe r3_freshwrap).
        let prefix = format!(
            "  rule `{}': fresh public constants are not allowed:",
            r.name
        );
        entries.push(fill_after_prefix(&prefix, &lits, 4, FILL_WIDTH));
    }
    if entries.is_empty() {
        vec![]
    } else {
        vec![WfError::new(T_FRESH_PUB, entries.join("\n  \n"))]
    }
}

// ---------------------------------------------------------------------------
// 5. Reserved prefixes (diff mode only)
// ---------------------------------------------------------------------------
// Fact names beginning with `DiffIntr`/`DiffProto` are reserved for the diff
// translation. Only emitted for diff-mode theories (observed: silent otherwise).
pub fn reserved_prefixes(thy: &Theory) -> Vec<WfError> {
    if !thy.is_diff {
        return vec![];
    }
    let mut entries: Vec<String> = Vec::new();
    for r in protocol_rules(thy) {
        // Order of collection: premises, actions, conclusions (probe rp_multi).
        let mut hits: Vec<&Fact> = Vec::new();
        for facts in [&r.premises, &r.actions, &r.conclusions] {
            for f in facts {
                if RESERVED_PREFIXES.iter().any(|p| f.name.starts_with(p)) {
                    hits.push(f);
                }
            }
        }
        if hits.is_empty() {
            continue;
        }
        let header = fill_words(&reserved_prefix_header_words(&r.name), 2, FILL_WIDTH);
        let blocks: Vec<String> = hits
            .iter()
            .map(|f| {
                let m = if f.persistent { "Persistent" } else { "Linear" };
                format!(
                    "    {}\n    (ProtoFact {} \"{}\" {},{},{})",
                    pp_fact(f),
                    m,
                    f.name,
                    f.args.len(),
                    f.args.len(),
                    m
                )
            })
            .collect();
        entries.push(format!("{}\n  \n{}", header, blocks.join("\n  \n")));
    }
    if entries.is_empty() {
        vec![]
    } else {
        vec![WfError::new(T_RESERVED_PREFIX, entries.join("\n  \n"))]
    }
}

fn reserved_prefix_header_words(rule: &str) -> Vec<String> {
    [
        "The",
        "Rule",
        &format!("`{}'", rule),
        "contains",
        "facts",
        "with",
        "reserved",
        "prefixes",
        "('DiffIntr',",
        "'DiffProto')",
        "inside",
        "names:",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

// ---------------------------------------------------------------------------
// 11 & 12. Left rule / Right rule (diff mode only)
// ---------------------------------------------------------------------------
// A diff rule may carry explicit `left`/`right` projections. Each explicit
// projection must equal the corresponding projection of the parent diff rule
// (diff(a,b) -> a on the left, -> b on the right). When it differs, the rule is
// "inconsistent". For a single rule the left projection is checked first and,
// if inconsistent, the right is not reported (observed via probe diff_both).
pub fn diff_left_right(thy: &Theory) -> Vec<WfError> {
    if !thy.is_diff {
        return vec![];
    }
    let mut left_entries: Vec<String> = Vec::new();
    let mut right_entries: Vec<String> = Vec::new();
    for r in protocol_rules(thy) {
        if let Some((left, right)) = &r.left_right {
            if !rule_matches_projection(r, left, true) {
                left_entries.push(inconsistent_entry("left", left, r));
            } else if !rule_matches_projection(r, right, false) {
                right_entries.push(inconsistent_entry("right", right, r));
            }
        }
    }
    let mut out = Vec::new();
    if !left_entries.is_empty() {
        out.push(WfError::new(T_LEFT, left_entries.join("\n  \n")));
    }
    if !right_entries.is_empty() {
        out.push(WfError::new(T_RIGHT, right_entries.join("\n  \n")));
    }
    out
}

fn inconsistent_entry(side: &str, explicit: &Rule, parent: &Rule) -> String {
    format!(
        "  Inconsistent {} rule\n{}\n  \n  w.r.t.\n  \n{}",
        side,
        indent_block(&pp_rule(explicit), 4),
        indent_block(&pp_rule(parent), 4)
    )
}

/// True iff the explicit rule's fact lists equal the parent rule's projection.
fn rule_matches_projection(parent: &Rule, explicit: &Rule, left: bool) -> bool {
    facts_pp(&project_facts(&parent.premises, left)) == facts_pp(&explicit.premises)
        && facts_pp(&project_facts(&parent.actions, left)) == facts_pp(&explicit.actions)
        && facts_pp(&project_facts(&parent.conclusions, left)) == facts_pp(&explicit.conclusions)
}

fn facts_pp(fs: &[Fact]) -> String {
    fs.iter().map(pp_fact).collect::<Vec<_>>().join(", ")
}

fn project_facts(fs: &[Fact], left: bool) -> Vec<Fact> {
    fs.iter().map(|f| project_fact(f, left)).collect()
}

fn project_fact(f: &Fact, left: bool) -> Fact {
    Fact {
        persistent: f.persistent,
        name: f.name.clone(),
        args: f.args.iter().map(|a| project_term(a, left)).collect(),
        annotations: f.annotations.clone(),
    }
}

/// Project a diff term to one side: `diff(a, b)` becomes `a` (left) or `b`
/// (right); all other terms are structurally mapped.
fn project_term(t: &Term, left: bool) -> Term {
    match t {
        Term::Diff(a, b) => project_term(if left { a } else { b }, left),
        Term::App(name, args) => {
            Term::App(name.clone(), args.iter().map(|a| project_term(a, left)).collect())
        }
        Term::Pair(args) => Term::Pair(args.iter().map(|a| project_term(a, left)).collect()),
        Term::AlgApp(name, a, b) => Term::AlgApp(
            name.clone(),
            Box::new(project_term(a, left)),
            Box::new(project_term(b, left)),
        ),
        Term::BinOp(op, a, b) => Term::BinOp(
            *op,
            Box::new(project_term(a, left)),
            Box::new(project_term(b, left)),
        ),
        Term::PatMatch(inner) => Term::PatMatch(Box::new(project_term(inner, left))),
        other => other.clone(),
    }
}

// ---------------------------------------------------------------------------
// 15. Lemma annotations
// ---------------------------------------------------------------------------
// An `exists-trace` lemma marked `reuse` is rejected (observed trigger).
pub fn lemma_annotations(thy: &Theory) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for it in &thy.items {
        if let TheoryItem::Lemma(l) = it {
            let is_reuse = l.attributes.iter().any(|a| matches!(a, LemmaAttr::Reuse));
            if is_reuse && l.trace_quantifier == TraceQuantifier::ExistsTrace {
                entries.push(format!(
                    "  Lemma `{}': cannot reuse 'exists-trace' lemmas",
                    l.name
                ));
            }
        }
    }
    if entries.is_empty() {
        vec![]
    } else {
        vec![WfError::new(T_LEMMA_ANNOT, entries.join("\n  \n"))]
    }
}

// ---------------------------------------------------------------------------
// 16. Multiplication restriction of rules
// ---------------------------------------------------------------------------
// A rule whose conclusions contain a multiplication (`*`) term is not
// multiplication restricted. The "After replacing reducible function symbols"
// rule renders identically to the original when the left-hand side has no
// reducible symbols to replace (the general replacement is a documented gap).
pub fn multiplication_restriction(thy: &Theory) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for r in protocol_rules(thy) {
        let mut mult_terms: Vec<String> = Vec::new();
        for f in &r.conclusions {
            for a in &f.args {
                collect_mult_terms(a, &mut mult_terms);
            }
        }
        if mult_terms.is_empty() {
            continue;
        }
        let rule_pp = indent_block(&pp_rule(r), 4);
        entries.push(format!(
            "  The following rule is not multiplication restricted:\n{rule}\n  \n  After replacing reducible function symbols in lhs with variables:\n{rule}\n  \n    Terms with multiplication:  {terms}",
            rule = rule_pp,
            terms = mult_terms.join(", ")
        ));
    }
    if entries.is_empty() {
        vec![]
    } else {
        vec![WfError::new(T_MULRESTRICT, entries.join("\n  \n"))]
    }
}

// ---------------------------------------------------------------------------
// check_if_lemmas_in_theory (secondary entry point; render UNVERIFIED)
// ---------------------------------------------------------------------------

/// Names of every lemma-like item declared in the theory.
pub fn theory_lemma_names(thy: &Theory) -> Vec<String> {
    thy.items
        .iter()
        .filter_map(|it| match it {
            TheoryItem::Lemma(l) => Some(l.name.clone()),
            TheoryItem::DiffLemma(l) => Some(l.name.clone()),
            TheoryItem::AccLemma(l) => Some(l.name.clone()),
            _ => None,
        })
        .collect()
}

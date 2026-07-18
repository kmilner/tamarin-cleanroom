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
pub const T_QUANT_SORTS: &str = "Quantifier sorts";
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

/// Builtin-fact normalization: fact names matching KU/KD/In/Out/Fr
/// case-insensitively (and the exact single letter `K`) denote the builtin
/// facts; `Ku(x)` reports as `!KU( x )` and `FR(x)` as `Fr( x )` (probes
/// t5_ku_lhs / t5_up_inout). Returns the canonical name and multiplicity.
fn canon_builtin(name: &str) -> Option<(&'static str, bool)> {
    match name.to_ascii_lowercase().as_str() {
        "k" => Some(("K", false)),
        "ku" => Some(("KU", true)),
        "kd" => Some(("KD", true)),
        "in" => Some(("In", false)),
        "out" => Some(("Out", false)),
        "fr" => Some(("Fr", false)),
        _ => None,
    }
}

/// The fact with its name/multiplicity rewritten to the canonical builtin
/// form when it matches one (otherwise unchanged).
fn canon_fact(f: &Fact) -> Fact {
    match canon_builtin(&f.name) {
        Some((name, persistent)) => Fact {
            persistent,
            name: name.to_string(),
            args: f.args.clone(),
            annotations: f.annotations.clone(),
        },
        None => f.clone(),
    }
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

/// Wrap each already-formatted per-finding entry as its own `WfError` for
/// `topic`, preserving order. `check_theory` returns ONE `WfError` per
/// INDIVIDUAL finding so the caller's list length equals the oracle's trailing
/// "<N> wellformedness check failed!" count; the render layer regroups
/// consecutive same-topic findings into the byte-exact block. See BEHAVIOR.md
/// "Per-topic finding-count law" for the granularity of each topic.
fn per_finding(topic: &'static str, entries: Vec<String>) -> Vec<WfError> {
    entries
        .into_iter()
        .map(|e| WfError::new(topic, e))
        .collect()
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
    // One finding per rule with unbound variables (probes deriv_2var: two
    // unbound vars in one rule -> one finding; deriv_2rule: two rules -> two).
    per_finding(T_UNBOUND, entries)
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

        // Group by (lowercased base name, index): `$x.1` and `x.2` are
        // different variables and never clash (probes s5_idx / s5_capdiff).
        // A group with >1 distinct variant (sort class + exact name) is a
        // conflict; suffix-sort and sigil spellings of the same sort are the
        // SAME variant (s5_suffix).
        type Variant = (i32, String); // (sort-class rank, exact name)
        let mut groups: Vec<((String, u64), Vec<Variant>)> = Vec::new();
        for v in &vars {
            let key = (v.name.to_lowercase(), v.idx);
            let variant = (class_rank(var_class(v)), v.name.clone());
            match groups.iter_mut().find(|(k, _)| *k == key) {
                Some((_, variants)) => {
                    if !variants.contains(&variant) {
                        variants.push(variant);
                    }
                }
                None => groups.push((key, vec![variant])),
            }
        }
        let mut conflicts: Vec<((String, u64), Vec<Variant>)> = groups
            .into_iter()
            .filter(|(_, vs)| vs.len() > 1)
            .map(|(k, mut vs)| {
                // Variants ordered by sort class ($ < ~ < msg < %), then by
                // exact name (probes s5_all4 / s5_crossname / s5_capord).
                vs.sort();
                (k, vs)
            })
            .collect();
        if conflicts.is_empty() {
            continue;
        }
        // Groups ordered by their (lowercased name, index) key (s5_groups).
        conflicts.sort_by(|a, b| a.0.cmp(&b.0));
        let mut body = format!("  rule `{}': ", r.name);
        for (i, ((_, idx), variants)) in conflicts.iter().enumerate() {
            if i > 0 {
                body.push_str("\n    ");
            }
            let rendered: Vec<String> = variants
                .iter()
                .map(|(rank, name)| variant_string(*rank, name, *idx))
                .collect();
            body.push_str(&format!("\n    {}. {}", i + 1, rendered.join(", ")));
        }
        rule_entries.push(body);
    }
    if rule_entries.is_empty() {
        return vec![];
    }
    // One finding per conflicting rule (probe sort_2rule: two rules -> two;
    // sort_2grp_1rule: two variant groups in one rule -> one). The fixed
    // "Possible reasons:" preamble is a topic-level heading, so it rides on the
    // FIRST rule's finding; the render layer joins the per-rule findings with
    // the standard finding separator, reproducing the single-preamble body.
    let header = "Possible reasons:\n1. Identifiers are case sensitive, i.e.,'x' and 'X' are considered to be different.\n2. The same holds for sorts:, i.e., '$x', 'x', and '~x' are considered to be different.\n";
    rule_entries
        .into_iter()
        .enumerate()
        .map(|(i, body)| {
            let msg = if i == 0 {
                format!("{}\n{}", header, body)
            } else {
                body
            };
            WfError::new(T_SORTS, msg)
        })
        .collect()
}

/// Report-order rank of a sort class in the variant listing: `$x, ~x, x, %x`
/// (probe s5_all4; Node placement unobserved - listed last).
fn class_rank(c: SortClass) -> i32 {
    match c {
        SortClass::Pub => 0,
        SortClass::Fresh => 1,
        SortClass::Msg => 2,
        SortClass::Nat => 3,
        SortClass::Node => 4,
    }
}

/// A variant rendered with its sort sigil and the group's index suffix
/// (suffix-sorted spellings render with the sigil too - probe s5_suffix2).
fn variant_string(rank: i32, name: &str, idx: u64) -> String {
    let sigil = match rank {
        0 => "$",
        1 => "~",
        3 => "%",
        4 => "#",
        _ => "",
    };
    if idx > 0 {
        format!("{}{}.{}", sigil, name, idx)
    } else {
        format!("{}{}", sigil, name)
    }
}

// ---------------------------------------------------------------------------
// 4. Reserved names
// ---------------------------------------------------------------------------
pub fn reserved_names(thy: &Theory) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for r in protocol_rules(thy) {
        // On the left/right the reserved set is {K,KU,KD}; in the middle
        // (actions) the I/O facts In/Out/Fr are reserved too (observed z9/z11).
        // Matching is via the builtin normalization (`Ku` hits as `!KU( x )` -
        // probe t5_ku_all).
        for (facts, phrase, middle) in [
            (&r.premises, "left-hand-side", false),
            (&r.actions, "the middle", true),
            (&r.conclusions, "the right-hand-side", false),
        ] {
            let hits: Vec<Fact> = facts
                .iter()
                .map(canon_fact)
                .filter(|f| is_reserved(&f.name) || (middle && is_special(&f.name)))
                .collect();
            if hits.is_empty() {
                continue;
            }
            let rendered: Vec<String> = hits.iter().map(pp_fact).collect();
            entries.push(format!(
                "  Rule `{}' contains facts with reserved names on {}:\n    {}",
                r.name,
                phrase,
                rendered.join(", ")
            ));
        }
    }
    // One finding per (rule, position) block (probe reserved_2fact_1side: two
    // reserved facts on one side -> one finding; issue515: 12 rule-side blocks
    // -> 12 findings).
    per_finding(T_RESERVED, entries)
}

// ---------------------------------------------------------------------------
// 5. Fr facts must only use a fresh- or a msg-variable
// ---------------------------------------------------------------------------
pub fn fr_facts(thy: &Theory) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for r in protocol_rules(thy) {
        for facts in [&r.premises, &r.conclusions] {
            for f in facts {
                let f = canon_fact(f);
                if f.name != "Fr" {
                    continue;
                }
                let ok = f.args.len() == 1
                    && matches!(
                        &f.args[0],
                        Term::Var(v) if matches!(v.sort, SortHint::Fresh | SortHint::Msg | SortHint::Untagged)
                    );
                if !ok {
                    entries.push(format!("  rule `{}' fact: {}", r.name, pp_fact(&f)));
                }
            }
        }
    }
    // One finding per offending Fr fact occurrence (probe fr_2fact_1rule: two
    // bad Fr facts in one rule -> two findings).
    per_finding(T_FR, entries)
}

// ---------------------------------------------------------------------------
// 6. Special facts (disallowed I/O facts in wrong position)
// ---------------------------------------------------------------------------
pub fn special_facts(thy: &Theory) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for r in protocol_rules(thy) {
        // Premise side: `Out` is disallowed (builtin-normalized; `FR(x)` in a
        // conclusion reports as `Fr( x )` - probe t5_up_inout).
        let lhs: Vec<String> = r
            .premises
            .iter()
            .map(canon_fact)
            .filter(|f| f.name == "Out")
            .map(|f| pp_fact(&f))
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
            .map(canon_fact)
            .filter(|f| f.name == "In" || f.name == "Fr")
            .map(|f| pp_fact(&f))
            .collect();
        if !rhs.is_empty() {
            entries.push(format!(
                "  rule `{}' uses disallowed facts on right-hand-side:\n    {}",
                r.name,
                rhs.join(", ")
            ));
        }
    }
    // One finding per (rule, side) block (probe special_2fact_1side + issue515:
    // test2's disallowed Out on the left and In on the right are two findings).
    per_finding(T_SPECIAL, entries)
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
fn show_haskell_fact(f: &Fact, stack: &[Binder]) -> String {
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
    stack: &mut Vec<Binder>,
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
                stack.push((v.name.clone(), v.idx, var_class(v)));
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
                let mut stack: Vec<Binder> = Vec::new();
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
                // Builtin facts normalize their spelling (`Ku` = `KU`), so
                // they never produce a capitalization conflict (issue515:
                // Ku/KU/Kd/KD yield no such topic).
                if canon_builtin(&f.name).is_some() {
                    continue;
                }
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
        // One item per distinct (label, owner, arity, render): two uses in
        // the same owner at the same arity are BOTH listed when their raw
        // renders differ (probe t5_lemdup: `Bound 3,2,1` and `Bound 4,3,2`).
        let mut seen: Vec<(&str, String, usize, String)> = Vec::new();
        let mut items: Vec<String> = Vec::new();
        for u in us {
            let k = (u.label, u.owner.clone(), u.arity, u.render.clone());
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
        let mut seen: Vec<(&str, String, bool, String)> = Vec::new();
        let mut items: Vec<String> = Vec::new();
        for u in us {
            let k = (u.label, u.owner.clone(), u.persistent, u.render.clone());
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
    // The builtin facts KU/KD/In/Out/Fr (matched via normalization) do not
    // participate; the reserved proto-fact `K` DOES - a K-only premise is
    // listed (issue527 target: `factName `K'` with suggestion `F`).
    let excluded = |name: &str| matches!(canon_builtin(name), Some((c, _)) if c != "K");
    let mut lhs: Vec<FactId> = Vec::new();
    let mut rhs: Vec<FactId> = Vec::new();
    for r in protocol_rules(thy) {
        for f in &r.premises {
            if excluded(&f.name) {
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
            if excluded(&f.name) {
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

    // Every LHS occurrence whose fact identity (name, arity, multiplicity) is
    // absent from every RHS is listed, in source order, WITHOUT deduplication:
    // one entry per premise occurrence, so a fact repeated in a single rule and
    // the same fact reused across several rules each contribute their own entry
    // (probes lhs1/lhs2/lhs3).
    let mut entries: Vec<String> = Vec::new();
    for f in &lhs {
        if rhs_idents.contains(&f.ident()) {
            continue;
        }
        let mut line = f.render();
        if let Some(sugg) = nearest_rhs(&f.name, &rhs) {
            line.push_str(&format!(". Perhaps you want to use the fact {}", sugg.render()));
        }
        entries.push(line);
    }
    if entries.is_empty() {
        return vec![];
    }
    // Item numbers are right-aligned to the widest index with a two-space
    // margin ("   1." ... "  10." - probe t5_align / ble & mesh targets).
    let w = entries.len().to_string().len();
    let items: Vec<String> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| format!("  {:>w$}. {}", i + 1, e, w = w))
        .collect();
    vec![WfError::new(T_LHSRHS, items.join("\n  \n"))]
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
    // Each DISTINCT spelling is attributed to the rule of its FIRST
    // occurrence (issue527 target: 'second' is listed only for rule One even
    // though later rules use it too).
    let mut first_rule: Vec<(String, String)> = Vec::new(); // (name, first rule)
    for (name, rule) in &pairs {
        if !first_rule.iter().any(|(n, _)| n == name) {
            first_rule.push((name.clone(), rule.clone()));
        }
    }
    // Group the distinct spellings by lowercased name; conflicting groups are
    // reported sorted by that key (issue527: 'first' group before 'second').
    let mut groups: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for (name, rule) in &first_rule {
        let key = name.to_lowercase();
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, v)) => v.push((name.clone(), rule.clone())),
            None => groups.push((key, vec![(name.clone(), rule.clone())])),
        }
    }
    groups.retain(|(_, v)| v.len() > 1);
    groups.sort_by(|a, b| a.0.cmp(&b.0));
    let mut items: Vec<String> = Vec::new();
    for (_, mut names) in groups {
        // Spellings sorted; consecutive spellings sharing their first rule
        // merge into one `rule "R":  name 'a', 'b'` segment.
        names.sort();
        let mut segs: Vec<(String, Vec<String>)> = Vec::new();
        for (name, rule) in names {
            match segs.last_mut() {
                Some((r, ns)) if *r == rule => ns.push(name),
                _ => segs.push((rule, vec![name])),
            }
        }
        let locs: Vec<String> = segs
            .into_iter()
            .map(|(rule, ns)| {
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
// Formula-variable sorts and sort-aware binding
// ---------------------------------------------------------------------------
// Formula quantifiers bind sorted variables. The oracle treats a bound variable
// and a use as the same variable only when they share a name AND a sort class,
// so a temporal binder `#x` does NOT bind a message-position use `x`. The parser
// tags an un-annotated message-position variable as `Untagged` and an annotated
// one as `Msg`; the oracle treats those as one variable, so both collapse to the
// `Msg` class here (this is what keeps a quantified message variable bound to its
// uses across the parser's tag differences).

/// Sort equivalence class of a formula variable, used for binding and guarding.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum SortClass {
    Msg,
    Pub,
    Fresh,
    Node,
    Nat,
}

fn sort_class(s: SortHint) -> SortClass {
    match s {
        SortHint::Msg | SortHint::Untagged | SortHint::Suffix(SuffixSort::Msg) => SortClass::Msg,
        SortHint::Pub | SortHint::Suffix(SuffixSort::Pub) => SortClass::Pub,
        SortHint::Fresh | SortHint::Suffix(SuffixSort::Fresh) => SortClass::Fresh,
        SortHint::Node | SortHint::Suffix(SuffixSort::Node) => SortClass::Node,
        SortHint::Nat | SortHint::Suffix(SuffixSort::Nat) => SortClass::Nat,
    }
}

fn var_class(v: &VarSpec) -> SortClass {
    sort_class(v.sort)
}

/// A binder on the quantifier stack: the bound variable's name, numeric index
/// and sort class. The index participates in variable identity (probe g5_idx:
/// a binder `y` does not bind a use `y.1`).
type Binder = (String, u64, SortClass);

/// The identity key of a formula variable: name, index and sort class.
fn formula_var_key(v: &VarSpec) -> (String, u64, SortClass) {
    (v.name.clone(), v.idx, var_class(v))
}

// ---------------------------------------------------------------------------
// Quantifier sorts (variables quantified over a disallowed sort)
// ---------------------------------------------------------------------------
// Quantifying over a public or fresh variable is rejected; message, temporal
// (node) and natural-number quantifiers are allowed. Each offending variable is
// reported as a Haskell `(name, LSort)` tuple, collected in binding order.

fn wrong_lsort(s: SortHint) -> Option<&'static str> {
    match sort_class(s) {
        SortClass::Pub => Some("LSortPub"),
        SortClass::Fresh => Some("LSortFresh"),
        SortClass::Msg | SortClass::Node | SortClass::Nat => None,
    }
}

/// Collect `(name,LSort)` tokens for every quantified variable of a disallowed
/// sort, visiting outer binders before inner ones (binding order).
fn wrong_quant_sorts(f: &Formula, out: &mut Vec<String>) {
    match f {
        Formula::Forall(vs, g) | Formula::Exists(vs, g) => {
            for v in vs {
                if let Some(ls) = wrong_lsort(v.sort) {
                    out.push(format!("(\"{}\",{})", v.name, ls));
                }
            }
            wrong_quant_sorts(g, out);
        }
        Formula::Not(g) => wrong_quant_sorts(g, out),
        Formula::And(a, b)
        | Formula::Or(a, b)
        | Formula::Implies(a, b)
        | Formula::Iff(a, b) => {
            wrong_quant_sorts(a, out);
            wrong_quant_sorts(b, out);
        }
        _ => {}
    }
}

fn quantifier_sorts_entry(entity: &str, name: &str, f: &Formula) -> Option<String> {
    let mut tokens = Vec::new();
    wrong_quant_sorts(f, &mut tokens);
    if tokens.is_empty() {
        return None;
    }
    let prefix = format!("  {} `{}' uses quantifiers with wrong sort:", entity, name);
    Some(fill_after_prefix(&prefix, &tokens, 4, FILL_WIDTH))
}

// ---------------------------------------------------------------------------
// Formula terms (ill-formed terms in lemma / restriction formulas)
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

/// De Bruijn index of `(name, idx, class)` on the binder stack (outermost
/// pushed first): the innermost binder matching name, index AND class is 0.
/// `None` if no binder matches (the variable is free). Every quantified
/// variable - including temporals and sort-mismatched ones - occupies one
/// stack slot, so the index counts all binders between the use and its
/// matching binder.
fn debruijn_index(stack: &[Binder], name: &str, idx: u64, class: SortClass) -> Option<usize> {
    stack
        .iter()
        .rposition(|(n, i, c)| n == name && *i == idx && *c == class)
        .map(|pos| stack.len() - 1 - pos)
}

/// Render a term in the oracle's raw "wrong form" representation.
fn show_wf_term(t: &Term, stack: &[Binder]) -> String {
    match t {
        Term::Var(v) => match debruijn_index(stack, &v.name, v.idx, var_class(v)) {
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
fn show_wf_pair(items: &[Term], stack: &[Binder]) -> String {
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
fn term_is_ill_formed(
    t: &Term,
    stack: &[Binder],
    reducible: &std::collections::BTreeSet<String>,
) -> bool {
    match t {
        Term::Var(v) => debruijn_index(stack, &v.name, v.idx, var_class(v)).is_none(), // free -> ill
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
/// reports `Free y` twice). Every quantified variable pushes a `(name, class)`
/// binder, so uses are matched sort-aware.
fn collect_ill_terms(
    f: &Formula,
    stack: &mut Vec<Binder>,
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
                stack.push((v.name.clone(), v.idx, var_class(v)));
            }
            collect_ill_terms(g, stack, reducible, out);
            for _ in 0..n {
                stack.pop();
            }
        }
    }
}

fn ill_terms(f: &Formula, reducible: &std::collections::BTreeSet<String>) -> Vec<String> {
    let mut stack: Vec<Binder> = Vec::new();
    let mut terms: Vec<String> = Vec::new();
    collect_ill_terms(f, &mut stack, reducible, &mut terms);
    terms
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

/// Standalone "Formula terms" check: reports ill-formed terms (free variables
/// and applications of any symbol in `reducible`) in every lemma (source order)
/// then every restriction (source order), merged into one topic block.
pub fn formula_terms_reducible(
    thy: &Theory,
    reducible: &std::collections::BTreeSet<String>,
) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for (entity, name, formula, _) in formula_items(thy) {
        let terms = ill_terms(formula, reducible);
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
/// reducible symbols). The reducible-aware entry point is
/// `formula_terms_reducible`.
pub fn formula_terms(thy: &Theory) -> Vec<WfError> {
    formula_terms_reducible(thy, &std::collections::BTreeSet::new())
}

// ---------------------------------------------------------------------------
// Formula guardedness (decision procedure over the guarded fragment)
// ---------------------------------------------------------------------------
// A LEMMA formula must be convertible to guarded form (an unguarded RESTRICTION
// is a fatal error, not a warning, so this check is lemma-only). Directly
// nested quantifiers of the same kind FUSE into one binder list before the
// check (probes g5_e_nest / g5_a_nest; the report renders the fused form -
// g5_a_nest_noimpl). A universal is guarded only as `guard ==> rest`; an
// existential's guard region is its whole body. Within the guard region a
// variable is guarded (resolved) by
//   1. occurring anywhere inside an ACTION atom reachable through
//      conjunctions (all action atoms are collected first - g5_e_actorder), or
//   2. an EQUALITY atom reachable through conjunctions, processed in a SINGLE
//      left-to-right pass (g5_e_eqchain vs g5_e_revchain): when one side of
//      the equality contains no unresolved quantified variables, every
//      unresolved quantified variable of the other side becomes resolved
//      (g5_e_eqbare/g5_e_eqinner/g5_e_eqpair; side-based, not unification -
//      g5_e_unif).
// Disjunction, negation, implication, ordering/subterm/last atoms and nested
// quantifiers contribute no guards. The first failing quantifier (antecedent
// before consequent, left before right) is reported together with the whole
// lemma formula.

/// A guardedness failure and its reason.
enum GuardFail {
    /// A quantifier binds variables not guarded by an action atom.
    Unguarded(Vec<VarSpec>),
    /// A universal quantifier's body is not a top-level implication.
    NoImplication,
}

/// Fuse directly nested quantifiers of the same kind (`∀x.∀y.φ` -> `∀x y.φ`),
/// recursively over the whole formula. The guardedness decision AND the
/// report's formula rendering both operate on the fused form.
fn fuse_quantifiers(f: &Formula) -> Formula {
    match f {
        Formula::Forall(vs, g) => {
            let mut vars = vs.clone();
            let mut body = fuse_quantifiers(g);
            while let Formula::Forall(vs2, g2) = body {
                vars.extend(vs2);
                body = *g2;
            }
            Formula::Forall(vars, Box::new(body))
        }
        Formula::Exists(vs, g) => {
            let mut vars = vs.clone();
            let mut body = fuse_quantifiers(g);
            while let Formula::Exists(vs2, g2) = body {
                vars.extend(vs2);
                body = *g2;
            }
            Formula::Exists(vars, Box::new(body))
        }
        Formula::Not(g) => Formula::Not(Box::new(fuse_quantifiers(g))),
        Formula::And(a, b) => Formula::And(
            Box::new(fuse_quantifiers(a)),
            Box::new(fuse_quantifiers(b)),
        ),
        Formula::Or(a, b) => Formula::Or(
            Box::new(fuse_quantifiers(a)),
            Box::new(fuse_quantifiers(b)),
        ),
        Formula::Implies(a, b) => Formula::Implies(
            Box::new(fuse_quantifiers(a)),
            Box::new(fuse_quantifiers(b)),
        ),
        Formula::Iff(a, b) => Formula::Iff(
            Box::new(fuse_quantifiers(a)),
            Box::new(fuse_quantifiers(b)),
        ),
        other => other.clone(),
    }
}

/// Collect the variables of every ACTION atom reachable through conjunctions of
/// `f`. These are the variables a guard binds outright.
fn collect_guard_vars(f: &Formula, out: &mut Vec<VarSpec>) {
    match f {
        Formula::Atom(Atom::Action(fact, t)) => {
            collect_fact_vars(fact, out);
            collect_term_vars(t, out);
        }
        Formula::And(a, b) => {
            collect_guard_vars(a, out);
            collect_guard_vars(b, out);
        }
        _ => {}
    }
}

/// Collect the EQUALITY atoms reachable through conjunctions of `f`, in
/// left-to-right source order.
fn collect_guard_eqs<'a>(f: &'a Formula, out: &mut Vec<(&'a Term, &'a Term)>) {
    match f {
        Formula::Atom(Atom::Eq(l, r)) => out.push((l, r)),
        Formula::And(a, b) => {
            collect_guard_eqs(a, out);
            collect_guard_eqs(b, out);
        }
        _ => {}
    }
}

type VKey = (String, u64, SortClass);

/// The subset of `current` (the quantifier's own variables) resolved by the
/// guard region `f`: first every conjunction-reachable action atom resolves
/// all of its variables, then the conjunction-reachable equalities are
/// processed once, left to right - an equality whose one side is clean (has no
/// unresolved current variables) resolves the current variables of its other
/// side.
fn resolved_guard_keys(
    f: &Formula,
    current: &std::collections::HashSet<VKey>,
) -> std::collections::HashSet<VKey> {
    let mut resolved: std::collections::HashSet<VKey> = std::collections::HashSet::new();
    let mut avs = Vec::new();
    collect_guard_vars(f, &mut avs);
    for v in &avs {
        let k = formula_var_key(v);
        if current.contains(&k) {
            resolved.insert(k);
        }
    }
    let mut eqs = Vec::new();
    collect_guard_eqs(f, &mut eqs);
    for (l, r) in eqs {
        let unresolved_side = |t: &Term| -> Vec<VKey> {
            let mut vs = Vec::new();
            collect_term_vars(t, &mut vs);
            vs.iter()
                .map(formula_var_key)
                .filter(|k| current.contains(k) && !resolved.contains(k))
                .collect()
        };
        let lu = unresolved_side(l);
        let ru = unresolved_side(r);
        if lu.is_empty() && !ru.is_empty() {
            resolved.extend(ru);
        } else if ru.is_empty() && !lu.is_empty() {
            resolved.extend(lu);
        }
    }
    resolved
}

/// Quantified variables of `vs` not resolved by the guard region `region`, in
/// binding order.
fn unguarded_vars(vs: &[VarSpec], region: &Formula) -> Vec<VarSpec> {
    let current: std::collections::HashSet<VKey> = vs.iter().map(formula_var_key).collect();
    let resolved = resolved_guard_keys(region, &current);
    vs.iter()
        .filter(|v| !resolved.contains(&formula_var_key(v)))
        .cloned()
        .collect()
}

/// Return the first guardedness failure and the failing quantifier subformula.
/// Expects a quantifier-fused formula (see [`fuse_quantifiers`]).
fn find_guard_failure(f: &Formula) -> Option<(GuardFail, Formula)> {
    match f {
        Formula::Forall(vs, body) => match &**body {
            Formula::Implies(guard, rest) => {
                let unguarded = unguarded_vars(vs, guard);
                if !unguarded.is_empty() {
                    return Some((GuardFail::Unguarded(unguarded), f.clone()));
                }
                find_guard_failure(guard).or_else(|| find_guard_failure(rest))
            }
            _ => Some((GuardFail::NoImplication, f.clone())),
        },
        Formula::Exists(vs, body) => {
            let unguarded = unguarded_vars(vs, body);
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

/// The guardedness report entry for one lemma, or `None` when it is guarded.
fn guardedness_entry(name: &str, f: &Formula) -> Option<String> {
    let f = &fuse_quantifiers(f);
    let (fail, sub) = find_guard_failure(f)?;
    let reason = match fail {
        GuardFail::Unguarded(vars) => {
            let vs: Vec<String> = vars.iter().map(|v| format!("'{}'", pp_var(v))).collect();
            format!("unguarded variable(s) {} in the subformula", vs.join(", "))
        }
        GuardFail::NoImplication => {
            "universal quantifier without toplevel implication".to_string()
        }
    };
    // The formula is embedded as `      "..."`: its first character sits at
    // column 7 (after six spaces and the quote).
    let pp_sub = crate::formula::pp_formula_wrapped(&sub, 7);
    let pp_whole = crate::formula::pp_formula_wrapped(f, 7);
    Some(format!(
        "  Lemma `{}' cannot be converted to a guarded formula:\n    {}\n      \"{}\"\n    in the formula\n      \"{}\"",
        name, reason, pp_sub, pp_whole
    ))
}

/// Standalone guardedness check over every lemma, merged into one topic block.
pub fn formula_guardedness(thy: &Theory) -> Vec<WfError> {
    let mut entries: Vec<String> = Vec::new();
    for it in &thy.items {
        if let TheoryItem::Lemma(l) = it {
            if let Some(e) = guardedness_entry(&l.name, &l.formula) {
                entries.push(e);
            }
        }
    }
    if entries.is_empty() {
        vec![]
    } else {
        vec![WfError::new(T_GUARD, entries.join("\n  \n"))]
    }
}

// ---------------------------------------------------------------------------
// Formula-check bundle (Quantifier sorts, Formula terms, Formula guardedness)
// ---------------------------------------------------------------------------
// The three per-formula checks are run together, item by item, in the oracle's
// checking order: every lemma (source order) first, then every restriction
// (source order). For each item the three topics are emitted in the order
// Quantifier sorts, Formula terms, Formula guardedness (guardedness for lemmas
// only). Consecutive entries sharing a topic merge under a single header; a
// topic that recurs after an intervening different topic starts a fresh block
// (observed: L1=QS, L2=FT, L3=QS renders as three blocks).

/// The formula-bearing items in checking order: (entity, name, formula,
/// is_lemma). Lemmas come before restrictions regardless of source order.
fn formula_items(thy: &Theory) -> Vec<(&'static str, &str, &Formula, bool)> {
    let mut out: Vec<(&'static str, &str, &Formula, bool)> = Vec::new();
    for it in &thy.items {
        if let TheoryItem::Lemma(l) = it {
            out.push(("Lemma", l.name.as_str(), &l.formula, true));
        }
    }
    for it in &thy.items {
        if let TheoryItem::Restriction(r) | TheoryItem::LegacyAxiom(r) = it {
            out.push(("Restriction", r.name.as_str(), &r.formula, false));
        }
    }
    out
}

/// Run the Quantifier-sorts / Formula-terms / Formula-guardedness bundle over
/// all lemmas and restrictions, producing ONE `WfError` per emitted finding in
/// report order (per formula item for Quantifier sorts and Formula terms, per
/// lemma for Formula guardedness - probes qs_2lem / ft_2lem / guard_2lem, each
/// two items -> two findings). The render layer merges CONSECUTIVE same-topic
/// findings back into one block, so an interleaved sequence such as QS, FT, QS
/// still renders as three separate blocks.
pub fn formula_reports(
    thy: &Theory,
    reducible: &std::collections::BTreeSet<String>,
) -> Vec<WfError> {
    let mut out: Vec<WfError> = Vec::new();
    for (entity, name, formula, is_lemma) in formula_items(thy) {
        if let Some(e) = quantifier_sorts_entry(entity, name, formula) {
            out.push(WfError::new(T_QUANT_SORTS, e));
        }
        let terms = ill_terms(formula, reducible);
        if !terms.is_empty() {
            out.push(WfError::new(T_FORMULA_TERMS, formula_terms_entry(entity, name, &terms)));
        }
        if is_lemma {
            if let Some(e) = guardedness_entry(name, formula) {
                out.push(WfError::new(T_GUARD, e));
            }
        }
    }
    out
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
    // One finding per distinct non-nat-in-nat-context entry (kept per-entry to
    // mirror the grouped-list topics; not directly probed - see BEHAVIOR.md).
    per_finding(T_NAT, entries)
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
    // An equation is subterm convergent when its RHS is a subterm of its LHS
    // or GROUND (no variables - probes t5_sub_ground/t5_sub_groundapp). The
    // flagged equations are listed sorted by their rendered form, not source
    // order (t5_sub_order*; mesh: k1..k4 before s1), and wide equations wrap
    // via the equation layout engine (ble f6 / mesh k2).
    let mut bad: Vec<(String, String)> = Vec::new(); // (flat sort key, rendered)
    for it in &thy.items {
        if let TheoryItem::Equations { convergent, eqs } = it {
            if *convergent {
                continue; // user asserted convergence
            }
            for eq in eqs {
                if !is_subterm(&eq.rhs, &eq.lhs) && term_has_vars(&eq.rhs) {
                    let key = format!("{} = {}", pp_term(&eq.lhs), pp_term(&eq.rhs));
                    bad.push((key, crate::pretty::pp_equation(&eq.lhs, &eq.rhs)));
                }
            }
        }
    }
    if bad.is_empty() {
        return vec![];
    }
    bad.sort();
    let bad: Vec<String> = bad.into_iter().map(|(_, r)| r).collect();
    let intro = "  User-defined equations must be convergent and have the finite variant property. The following equations are not subterm convergent. If you are sure that the set of equations is nevertheless convergent and has the finite variant property, you can ignore this warning and continue ";
    let manual = " For more information, please refer to the manual : https://tamarin-prover.com/manual/master/book/010_modeling-issues.html ";
    let msg = format!("{}\n\n{}\n   \n{}", intro, bad.join("\n"), manual);
    vec![WfError::new(T_SUBTERM, msg)]
}

/// Does the term contain any variable? (A ground RHS is convergent.)
fn term_has_vars(t: &Term) -> bool {
    let mut vs = Vec::new();
    collect_term_vars(t, &mut vs);
    !vs.is_empty()
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
    // One finding per rule that uses fresh public constants (probe
    // freshpub_2rule: two rules -> two; freshpub_2const_1rule: two constants in
    // one rule -> one).
    per_finding(T_FRESH_PUB, entries)
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
    // One finding per rule with reserved-prefix facts (diff mode only).
    per_finding(T_RESERVED_PREFIX, entries)
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
    // One finding per inconsistent rule on each side (diff mode only); the
    // Left-rule findings precede the Right-rule findings.
    let mut out = per_finding(T_LEFT, left_entries);
    out.extend(per_finding(T_RIGHT, right_entries));
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
    // One finding per offending lemma (probe lemanno_2lem: two reuse
    // exists-trace lemmas -> two findings).
    per_finding(T_LEMMA_ANNOT, entries)
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
    // One finding per non-multiplication-restricted rule (probe
    // multrestrict_2rule_dh: two rules -> two findings).
    per_finding(T_MULRESTRICT, entries)
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

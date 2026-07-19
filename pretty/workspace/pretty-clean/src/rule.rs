//! R2 — rule rendering: the whole rule block, byte-identical to the echo.
//!
//! Every layout decision is oracle-pinned (see workspace/BEHAVIOR.md "Rule
//! blocks" for probe provenance). Shape summary:
//!
//! ```text
//! rule (modulo E) Name[color=#abcdef, no_derivcheck, issapicrule,
//!                      role='r']:
//!    [ Prem( x ), !Persistent( y )[+, no_precomp] ]
//!   --[ Act( x ) ]->
//!    [ Concl( x ) ]
//!
//!   // loop breakers: [0,1]
//!   /* has exactly the trivial AC variant */
//! ```
//!
//! * body = `nest 3 (sep [prems, nest (-1) arrow, concls])` — one line when
//!   it fits, otherwise premises/conclusions at indent 3 and the arrow row at
//!   indent 2;
//! * each bracket group = `sep ["[", fsep facts, "]"]` (`[ ]` when empty);
//!   the empty-action arrow is the literal `-->`;
//! * a fact = `sep [name<>"(" <+> fsep args, ")"]` (`Name( )` when nullary)
//!   with the annotation suffix attached after the `)`;
//! * the trailing comment is either the trivial-variant one-liner or the
//!   `/* rule (modulo AC) … variants (modulo AC) … */` block with numbered
//!   substitution groups: number right-aligned to the widest index, each
//!   substitution `lhs $$ nest 6 ("= " <> rhs)` — the HughesPJ overlap pads
//!   a short lhs to column 6 and pushes `= rhs` to its own line otherwise.

use crate::ast::{AcVariants, Fact, FactAnnotation, Rule, RuleAttr, Term};
use crate::doc::{
    above_op, above_plus, beside_op, beside_space, fsep, nest, punctuate,
    render_with, sep, vcat, Doc,
};
use crate::term::{self, RIBBON, WIDTH};
use crate::web::{hl_comment, hl_kw, hl_op_text, hl_wrap, w_char as char, w_text as text};

/// One fact: `Name( arg, … )`, persistent prefixed with `!`, annotations
/// attached after the closing paren.
pub fn render_fact(f: &Fact) -> String {
    render_with(WIDTH, RIBBON, &fact_doc(f))
}

/// One whole rule block: header, body, blank line, loop-breaker annotation,
/// and the AC-variants comment (`None` → the trivial-variant one-liner).
pub fn render(rule: &Rule, variants: Option<&AcVariants>) -> String {
    render_with(WIDTH, RIBBON, &block_doc(rule, variants))
}

pub(crate) fn block_doc(rule: &Rule, variants: Option<&AcVariants>) -> Doc {
    let mut d = core_doc(rule);
    // `$+$` below the blank line: `$$` overlap would otherwise pull the
    // nested comment line up onto it.
    d = above_plus(d, text(""));
    if let Some(lb) = breaker_doc(&rule.loop_breakers) {
        d = above_plus(d, nest(2, &lb));
    }
    above_plus(d, nest(2, &comment_doc(variants)))
}

/// Header + body (shared by the toplevel block and the AC rule re-rendered
/// inside the variants comment; also the bare construction/deconstruction
/// message-pane rule form in web mode).
pub(crate) fn core_doc(rule: &Rule) -> Doc {
    above_op(header_doc(rule), nest(3, &body_doc(rule)))
}

// ── header ──────────────────────────────────────────────────────────────────

/// `rule (modulo E) Name:`, with the rendered attribute list in `[…]` before
/// the colon; attributes fill-wrap aligned after the `[`
/// (target:issue713, probe:p_rattr).
fn header_doc(rule: &Rule) -> Doc {
    // Keyword-prefix `rule ` (+ `(modulo E) `): `rule` and `modulo` are
    // `hl_keyword`-spanned in web mode, the `(`/`)`/annotation between them
    // plain; identity in batch (`rule (modulo E) `).
    let mut prefix = hl_kw("rule");
    if let Some(m) = &rule.modulo {
        prefix = beside_op(
            prefix,
            beside_op(
                text(" ("),
                beside_op(hl_kw("modulo"), text(&format!(" {m}) "))),
            ),
        );
    } else {
        prefix = beside_op(prefix, text(" "));
    }
    let attrs = attr_items(&rule.attributes);
    if attrs.is_empty() {
        beside_op(prefix, text(&format!("{}:", rule.name)))
    } else {
        beside_op(
            beside_op(
                beside_op(prefix, text(&format!("{}[", rule.name))),
                fsep(punctuate(
                    &char(','),
                    attrs.iter().map(|a| text(a)).collect(),
                )),
            ),
            text("]:"),
        )
    }
}

/// Canonical attribute list: color, process, no_derivcheck, issapicrule, role
/// — the last color/process/role declaration wins; external attributes are
/// dropped (probe:p_rattr, target:issue713; target:ct pins the SAPIC
/// `process` attribute).
///
/// The SAPIC `process="…"` attribute (target:ct, probe:p_process) renders
/// DOUBLE-quoted (unlike `role='…'`), between `color` and `no_derivcheck`, and
/// carries the process snippet VERBATIM: no escaping is applied and none is
/// observable (its text uses single-quoted string constants — `'proofOfID'` —
/// so no `"`/`\` ever appears; spaces, commas, `<>`, `()`, `;` all sit inside
/// the quotes as one unbreakable text token). Absent when the rule is not
/// SAPIC-generated (no `RuleAttr::Process`).
fn attr_items(attrs: &[RuleAttr]) -> Vec<String> {
    let mut color = None;
    let mut process = None;
    let mut no_derivcheck = false;
    let mut issapicrule = false;
    let mut role = None;
    for a in attrs {
        match a {
            RuleAttr::Color(c) => color = Some(c.clone()),
            RuleAttr::Process(p) => process = Some(p.clone()),
            RuleAttr::NoDerivCheck => no_derivcheck = true,
            RuleAttr::IsSapicRule => issapicrule = true,
            RuleAttr::Role(r) => role = Some(r.clone()),
            RuleAttr::External(..) => {}
        }
    }
    let mut items = Vec::new();
    if let Some(c) = color {
        items.push(format!("color=#{c}"));
    }
    if let Some(p) = process {
        items.push(format!("process=\"{p}\""));
    }
    if no_derivcheck {
        items.push("no_derivcheck".into());
    }
    if issapicrule {
        items.push("issapicrule".into());
    }
    if let Some(r) = role {
        items.push(format!("role='{r}'"));
    }
    items
}

// ── body ────────────────────────────────────────────────────────────────────

/// `sep [prems, nest (-1) arrow, concls]` under the rule's nest 3: one line
/// `[ … ] --[ … ]-> [ … ]` when it fits, otherwise three rows with the arrow
/// out-dented one column (targets cav13/NSLPK3, probe:t_wide).
fn body_doc(rule: &Rule) -> Doc {
    sep(vec![
        fact_list_doc(&rule.premises),
        nest(-1, &arrow_doc(&rule.actions)),
        fact_list_doc(&rule.conclusions),
    ])
}

/// `-->` when there are no actions, otherwise the `--[ … ]->` bracket group
/// (probe:p_arr1 rule AR pins the arrow to the SAME nested-sep construction
/// as the premise/conclusion lists: actions stay beside `--[` while `]->`
/// drops alone).
fn arrow_doc(actions: &[Fact]) -> Doc {
    if actions.is_empty() {
        hl_op_text("-->")
    } else {
        bracket_group("--[", actions, "]->")
    }
}

fn fact_list_doc(facts: &[Fact]) -> Doc {
    bracket_group("[", facts, "]")
}

/// `sep [sep [open, fsep facts], close]` — a graded three-way layout
/// (probe:p_arr1; targets mesh ProvisionerWaitingUser vs DeviceWaitingUser,
/// NSLPK3 R_1/I_2, Tutorial Serv_1):
///   * everything on one line when `open facts close` fits (`[ f1, f2 ]`,
///     kept at exactly ribbon width — p_arr1 rule PR at 73);
///   * otherwise the closing bracket drops to the group's column; the
///     opening bracket keeps ALL the facts beside it iff they fit on that
///     line as a unit (mesh DeviceWaitingUser at exactly 73);
///   * otherwise the opening bracket is alone too and the facts fill-wrap
///     at the group's column between the brackets (NSLPK3 R_1: three facts
///     share a fill line; Serv_1: `--[` alone although the first action
///     would fit beside it).
fn bracket_group(open: &str, facts: &[Fact], close: &str) -> Doc {
    // The bracket/arrow glyphs (`[` `]` `--[` `]->`) are `hl_operator`-spanned
    // in web mode; the fact separators and the facts themselves stay plain.
    if facts.is_empty() {
        return sep(vec![hl_op_text(open), hl_op_text(close)]);
    }
    sep(vec![
        sep(vec![
            hl_op_text(open),
            fsep(punctuate(&char(','), facts.iter().map(fact_doc).collect())),
        ]),
        hl_op_text(close),
    ])
}

// ── facts ───────────────────────────────────────────────────────────────────

/// `Name( a, b )` — args fill-wrap aligned after `Name( `, the closing paren
/// drops to the fact's own column when the args are multi-line
/// (probe:t_wide); `Name( )` when nullary (target:mesh); `!` prefix for
/// persistent facts; annotation suffix attached directly after the paren.
/// Formula action atoms reuse this construction unchanged (probe:q_l3).
pub(crate) fn fact_doc(f: &Fact) -> Doc {
    let head = format!("{}{}(", if f.persistent { "!" } else { "" }, f.name);
    let core = if f.args.is_empty() {
        sep(vec![text(&head), char(')')])
    } else {
        sep(vec![
            beside_space(
                text(&head),
                fsep(punctuate(
                    &char(','),
                    f.args.iter().map(term::doc).collect(),
                )),
            ),
            char(')'),
        ])
    };
    match annotation_suffix(&f.annotations) {
        Some(s) => beside_op(core, text(&s)),
        None => core,
    }
}

/// `[+, -, no_precomp]` in canonical order regardless of source order
/// (probe:p_fann `[no_precomp,+]` → `[+, no_precomp]`; target:seqdfsneeded
/// `[no_precomp,-]` → `[-, no_precomp]`).
fn annotation_suffix(annotations: &[FactAnnotation]) -> Option<String> {
    if annotations.is_empty() {
        return None;
    }
    let mut sorted: Vec<FactAnnotation> = annotations.to_vec();
    sorted.sort();
    sorted.dedup();
    let spellings: Vec<&str> = sorted
        .iter()
        .map(|a| match a {
            FactAnnotation::SolveFirst => "+",
            FactAnnotation::SolveLast => "-",
            FactAnnotation::NoSources => "no_precomp",
        })
        .collect();
    Some(format!("[{}]", spellings.join(", ")))
}

// ── loop breakers ───────────────────────────────────────────────────────────

/// `// loop breaker: [0]` / `// loop breakers: [0,1]` — plural iff several,
/// indices comma-separated without spaces (probes c_loop, p_lb2).
fn breaker_doc(indices: &[usize]) -> Option<Doc> {
    if indices.is_empty() {
        return None;
    }
    let list = indices
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let noun = if indices.len() == 1 {
        "loop breaker"
    } else {
        "loop breakers"
    };
    // `hl_comment`-spanned in web mode — even the AC-side breaker inside a
    // variants comment gets its OWN nested comment span (corpus:
    // ISO_IEC9798_3_3 / TPM_Exclusive_Secrets).
    Some(hl_comment(&format!("// {noun}: [{list}]")))
}

// ── the AC-variants comment ─────────────────────────────────────────────────

/// `None` → the trivial one-liner; otherwise the comment block re-rendering
/// the AC rule, its numbered substitution groups, and its loop-breaker line
/// (probes p_var1, p_lbvar; targets cav13/Joux/CH07/mesh).
fn comment_doc(variants: Option<&AcVariants>) -> Doc {
    let Some(v) = variants else {
        return hl_comment("/* has exactly the trivial AC variant */");
    };
    // The whole `/* … */` block is ONE `hl_comment` span in web mode, with the
    // AC-rule re-render's own keyword/operator spans nested inside (identity in
    // batch).
    let mut d = above_op(text("/*"), core_doc(&v.ac_rule));
    if !v.substitutions.is_empty() {
        d = above_op(d, nest(2, &substitutions_doc(&v.substitutions)));
    }
    if let Some(lb) = breaker_doc(&v.ac_rule.loop_breakers) {
        d = above_op(d, nest(2, &lb));
    }
    d = above_op(d, text("*/"));
    hl_wrap("hl_comment", d)
}

/// `variants (modulo AC)` and the numbered groups, separated by lines of
/// bare indent (four spaces at the observed nesting — probe:p_var1 byte
/// check); numbers right-aligned to the widest index (`  1.` … `160.` —
/// targets cav13 (1 digit), CH07 (2), Joux (3)).
fn substitutions_doc(groups: &[Vec<(Term, Term)>]) -> Doc {
    let width = groups.len().to_string().len();
    // `variants` and `modulo` are `hl_keyword`-spanned in web mode, the rest of
    // the header plain; identity in batch (`variants (modulo AC)`).
    let variants_header = beside_op(
        hl_kw("variants"),
        beside_op(text(" ("), beside_op(hl_kw("modulo"), text(" AC)"))),
    );
    let mut rows = vec![variants_header];
    for (i, group) in groups.iter().enumerate() {
        if i > 0 {
            rows.push(text(""));
        }
        let prefix = format!("{:>width$}. ", i + 1);
        rows.push(beside_op(
            text(&prefix),
            vcat(group.iter().map(|(l, r)| substitution_doc(l, r)).collect()),
        ));
    }
    vcat(rows)
}

/// One substitution: `lhs $$ nest 6 ("= " <> rhs)` — the overlap pads a
/// short lhs to column 6 (`~lv2  = ~lv2.4`) and drops `= rhs` to its own
/// line at column 6 when the lhs is 6 columns or wider (probe:p_var1
/// `~longvariablenameone`); the rhs is a full R1 term with its own wrapping
/// (target:mesh multi-line substitution values).
fn substitution_doc(lhs: &Term, rhs: &Term) -> Doc {
    above_op(
        term::doc(lhs),
        nest(6, &beside_op(text("= "), term::doc(rhs))),
    )
}

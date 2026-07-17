//! Record-cell rendering (BEHAVIOR.md §3a, §3f): turning facts / rule instances
//! into the exact text that goes inside a record cell, including the observed
//! escaping, the fact-vs-function spacing, and the `\l`/`&nbsp;` line-wrapping
//! FORMAT.
//!
//! What is byte-exact here (verified against the corpus):
//!   * **Escaping** of record-label metacharacters `< > { } |`.
//!   * **Fact spacing** `Name( a, b )` (a space after `(` and before `)`) versus
//!     **function spacing** `f(a, b)` (no inner spaces) — mined over the corpus;
//!     a fact / relation symbol pads, an ordinary function application does not.
//!   * **Info-cell** shape `#<temporal> : <RuleName>` optionally followed by
//!     `[<action>, …]`.
//!   * The **wrap alignment**: a broken group's continuation lines are indented
//!     with `&nbsp;` runs to the column of the group's first element — i.e. just
//!     after `( ` for a fact, after `<` for a tuple, after `[` for an action list
//!     (verified across 188 192 wrapped cells: the indent always equals that
//!     first-element column). Physical segments are separated by `\l`.
//!
//! The wrap DECISION and the delimiter peel are byte-implemented by [`wrap_cell`],
//! pinned by a controlled live width sweep (BEHAVIOR.md §3f, QUERIES.log round 5):
//! a fact is laid out from its own column 0 with line width [`FILL_WIDTH`] = 87.
//! Two combinators, each byte-verified against captured probes:
//!   * a fact's **argument list** and a **tuple**'s elements use a greedy fill
//!     (`fillSep`): pack elements left-to-right, the `, ` separator trails the
//!     element it follows, and the next element wraps when it would pass column 87.
//!   * an info cell's **action list** uses a vertical `sep` (one action per line
//!     when the cell overflows).
//!
//! The **delimiter peel** at the fill-width boundary (byte-verified over the E-/W-
//! sweep fixtures): a tuple's `>` stays with the last element iff it fits, else it
//! peels onto its own line at the tuple's `<` column; a fact's `)` **always** peels
//! onto its own line at column 0 once the fact wraps (the padded ` )` space becomes
//! the break). An action list's `]` stays attached to the last action. The
//! continuation-line indent equals the broken group's first-element column
//! (`&nbsp;` runs). The one-element-lookahead observed earlier is subsumed: the
//! continuation packs greedily to the same width as the first line.
//!
//! Residual (characterised, not byte-implemented): [`wrap_cell`] triggers a wrap on
//! the cell's own flat width alone (correct for cells early on their record line).
//! When earlier cells push the absolute column high, tamarin's Wadler `group`/
//! `fits` decision wraps a cell **earlier** (its flat width alone can be well under
//! 87 — e.g. a 21-char fact deep in a wide record still peels its `)`), because
//! `fits` measures the flat rendering *plus the rest of the record line* against
//! the remaining width. Reproducing that needs the exact document tree; see
//! BEHAVIOR.md §3f.

use crate::term::Term;

/// Absolute per-fact line width of the record-cell pretty-printer, in columns,
/// measured from the functor at column 0. A fact whose flat rendering is `≤ 87`
/// columns is emitted on a single line; a wider one is broken (BEHAVIOR.md §3f).
/// Pinned by a live one-column width sweep that was invariant across functor
/// lengths (the boundary was always flat-width 87 fits / 88 breaks).
pub const FILL_WIDTH: usize = 87;

/// The wrap DECISION at the top level: a fact stays on a single record-cell line
/// iff its flat (un-wrapped) rendering is at most [`FILL_WIDTH`] columns wide.
/// `flat` is the fact's flat rendering (e.g. from [`Fact::render_flat`]); width is
/// counted in Unicode scalars, matching the observed column count. Verified
/// byte-exact at the boundary: an 87-column fact is single-line, an 88-column one
/// wraps (live probe, functor-length-invariant).
pub fn fits_one_line(flat: &str) -> bool {
    flat.chars().count() <= FILL_WIDTH
}

/// Escape the metacharacters that are special inside a graphviz record label:
/// `< > { } |` each get a leading backslash. Everything else (including single
/// quotes, `~ $ ^ * ⊕`, spaces) is literal. Observed in every record cell.
pub fn escape_record(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' | '>' | '{' | '}' | '|' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// A fact / relation atom occupying one premise or conclusion cell, e.g.
/// `!SessionKey( <…>, $A, H1 )`. `name` is the fact symbol (may start with `!`
/// for a persistent fact); `args` are its term arguments.
#[derive(Clone, Debug)]
pub struct Fact {
    pub name: String,
    pub args: Vec<Term>,
}

impl Fact {
    pub fn new(name: &str, args: Vec<Term>) -> Self {
        Fact { name: name.to_string(), args }
    }

    /// Flat (un-wrapped, un-escaped) surface rendering with **fact spacing**:
    /// `Name( a, b )`, and `Name( )` when it has no arguments. Argument terms use
    /// the ordinary term rendering (functions `f(a, b)` without inner spaces,
    /// tuples `<a, b>`). This matches every un-wrapped fact cell in the corpus.
    pub fn render_flat(&self) -> String {
        let inner: Vec<String> = self.args.iter().map(Term::render_full).collect();
        pad(&self.name, &inner)
    }

    /// Flat rendering with abbreviations applied to registered sub-terms
    /// (`table` maps a term's full rendering to its abbreviation name).
    pub fn render_flat_abbrev(&self, table: &std::collections::HashMap<String, String>) -> String {
        let inner: Vec<String> = self.args.iter().map(|t| t.render_abbrev(table)).collect();
        pad(&self.name, &inner)
    }
}

/// Fact spacing `Name( a, b )`, collapsing to `Name( )` for zero arguments
/// (observed: `NoA( )`, `!Semistate_1( )` — a single space between the parens).
fn pad(name: &str, args: &[String]) -> String {
    if args.is_empty() {
        format!("{}( )", name)
    } else {
        format!("{}( {} )", name, args.join(", "))
    }
}

/// The info cell of a rule instance: `#<temporal> : <rule>` plus, when the rule
/// has action facts, `[<action>, …]`. Matches the observed info-cell shape
/// (`#i : I_Complete[Complete( … ), …]`, or a bare `#vf.5 : Fresh`).
pub fn render_info(temporal: &str, rule: &str, actions: &[Fact]) -> String {
    if actions.is_empty() {
        format!("#{} : {}", temporal, rule)
    } else {
        let acts: Vec<String> = actions.iter().map(Fact::render_flat).collect();
        format!("#{} : {}[{}]", temporal, rule, acts.join(", "))
    }
}

/// Lay `items` out as a **fill** to a target `width`, starting at column
/// `open_col` (the alignment column of the group's first element). Returns the
/// physical lines *without* the `&nbsp;` indent — [`join_wrapped`] adds it.
///
/// Greedy fill: keep appending `sep`-joined items to the current line while it
/// fits in `width`; otherwise start a new line. This reproduces the observed
/// packing (several short arguments share a line, then a break) **given** the
/// width; the width itself is the documented gap.
pub fn fill(items: &[String], sep: &str, open_col: usize, width: usize) -> Vec<String> {
    if items.is_empty() {
        return vec![String::new()];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut col = open_col;
    for (i, it) in items.iter().enumerate() {
        let piece = if cur.is_empty() {
            it.clone()
        } else {
            format!("{}{}", sep, it)
        };
        if !cur.is_empty() && col + piece.chars().count() > width {
            lines.push(std::mem::take(&mut cur));
            col = open_col;
            cur.push_str(it);
            col += it.chars().count();
        } else {
            cur.push_str(&piece);
            col += piece.chars().count();
        }
        let _ = i;
    }
    lines.push(cur);
    lines
}

/// Greedy paragraph fill at the **known** width [`FILL_WIDTH`], reproducing the
/// observed record-cell packing: elements are laid left-to-right starting at
/// column `open_col`; the separator `sep` (e.g. `", "`) is emitted after each
/// element and **stays on the current line**, and the next element starts a new
/// line iff it would push past [`FILL_WIDTH`] from that point. This reproduces the
/// observed first-line packing exactly (e.g. an `Out( <'a01', … > )` whose tuple
/// overflows keeps eleven `'aNN'` elements plus the trailing `", "` on line 0,
/// then breaks). Continuation lines carry the [`FILL_WIDTH`]-wide `&nbsp;` indent
/// via [`join_wrapped`].
///
/// KNOWN RESIDUAL (BEHAVIOR.md §3f): tamarin's underlying `fsep`-style combinator
/// has a one-element lookahead that lets a *continuation* line hold one more
/// element than the first line at the same start column; this greedy pass packs
/// the first line exactly but a continuation line one element short of that
/// lookahead, and it does not model the closing-delimiter peel (`>`/`)` onto their
/// own aligned lines) that appears when the last element fills the line.
pub fn paragraph_fill(items: &[String], sep: &str, open_col: usize) -> Vec<String> {
    if items.is_empty() {
        return vec![String::new()];
    }
    let sep_w = sep.chars().count();
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut col = open_col;
    for (i, it) in items.iter().enumerate() {
        let w = it.chars().count();
        if i == 0 {
            cur.push_str(it);
            col += w;
        } else if col + w <= FILL_WIDTH {
            cur.push_str(it);
            col += w;
        } else {
            lines.push(std::mem::take(&mut cur));
            col = open_col;
            cur.push_str(it);
            col += w;
        }
        if i + 1 < items.len() {
            // The separator trails the element it follows and stays on the line;
            // the *next* element's fit is tested from past it.
            cur.push_str(sep);
            col += sep_w;
        }
    }
    lines.push(cur);
    lines
}

/// Join physical `lines` (a FILL of same-aligned elements, e.g. a broken action
/// list `[a1, a2]`) into a single record-cell string using the observed wrap
/// format: each physical line is followed by `\l`, and every continuation line is
/// prefixed by `open_col` `&nbsp;` entities (the verified alignment indent). The
/// trailing `\l` after the last line is part of the observed format (graphviz
/// left-justification). The caller escapes the line contents first (via
/// [`escape_record`]) — `\l` and `&nbsp;` are literal control sequences and must
/// not be escaped.
///
/// This reproduces the fill case where the closing delimiter stays attached to
/// the last element (`… )]`). A function/fact whose closing `)` breaks onto its
/// *own* line at the functor column is a distinct layout the pretty-printer's
/// break decision produces (a documented GAP), not modelled here.
pub fn join_wrapped(lines: &[String], open_col: usize) -> String {
    let indent = "&nbsp;".repeat(open_col);
    let mut out = String::new();
    for (i, ln) in lines.iter().enumerate() {
        if i > 0 {
            out.push_str(&indent);
        }
        out.push_str(ln);
        out.push_str("\\l");
    }
    out
}

/// Width of a rendered fragment in the pretty-printer's column model: Unicode
/// scalar count (matches the observed column boundary).
fn width(s: &str) -> usize {
    s.chars().count()
}

/// One physical line of a wrapped cell: its `&nbsp;` indent column and its
/// un-escaped text (escaping is applied when the lines are joined).
type PLine = (usize, String);

/// One fill element with the separator that precedes it, the indent to use if it
/// starts a new line, and whether a break before it is forced (`hard`).
struct Unit {
    sep: &'static str,
    text: String,
    indent: usize,
    hard: bool,
}

/// Greedy fill layout (BEHAVIOR.md §3f). Starting from `open_text` at column
/// `start_col`, place each unit's separator on the current line and then its text,
/// wrapping to the unit's indent when the text would pass [`FILL_WIDTH`] (or on a
/// `hard` unit). Returns the physical lines.
fn run_layout(open_text: &str, start_col: usize, units: &[Unit]) -> Vec<PLine> {
    let mut lines: Vec<PLine> = Vec::new();
    let mut cur = open_text.to_string();
    let mut col = start_col;
    let mut line_indent = 0usize;
    for u in units {
        cur.push_str(u.sep);
        col += width(u.sep);
        let w = width(&u.text);
        if u.hard || col + w > FILL_WIDTH {
            lines.push((line_indent, std::mem::take(&mut cur)));
            line_indent = u.indent;
            cur.push_str(&u.text);
            col = u.indent + w;
        } else {
            cur.push_str(&u.text);
            col += w;
        }
    }
    lines.push((line_indent, cur));
    lines
}

/// Split `s` at top-level occurrences of `", "` — respecting `()`/`<>`/`[]` nesting
/// and `'…'` quotes — into the comma-separated elements of a fill group.
fn split_top_commas(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut parts: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut in_quote = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if in_quote {
            cur.push(c);
            if c == '\'' {
                in_quote = false;
            }
            i += 1;
            continue;
        }
        match c {
            '\'' => {
                in_quote = true;
                cur.push(c);
                i += 1;
            }
            '(' | '<' | '[' => {
                depth += 1;
                cur.push(c);
                i += 1;
            }
            ')' | '>' | ']' => {
                depth -= 1;
                cur.push(c);
                i += 1;
            }
            ',' if depth == 0 && i + 1 < chars.len() && chars[i + 1] == ' ' => {
                parts.push(std::mem::take(&mut cur));
                i += 2;
            }
            _ => {
                cur.push(c);
                i += 1;
            }
        }
    }
    parts.push(cur);
    parts
}

fn is_tuple(s: &str) -> bool {
    s.starts_with('<') && s.ends_with('>')
}

/// Lay out a tuple `<e1, e2, …>` whose `<` sits at column `base_col`: fill the
/// elements (`fillSep`) at `base_col + 1`, and peel the `>` to its own line at
/// `base_col` iff it does not fit on the last element's line.
fn layout_tuple(inner: &str, base_col: usize) -> Vec<PLine> {
    let elems = split_top_commas(inner);
    let open_col = base_col + 1;
    let mut units: Vec<Unit> = Vec::with_capacity(elems.len() + 1);
    for (i, e) in elems.iter().enumerate() {
        units.push(Unit { sep: if i == 0 { "" } else { ", " }, text: e.clone(), indent: open_col, hard: false });
    }
    units.push(Unit { sep: "", text: ">".to_string(), indent: base_col, hard: false });
    run_layout("<", open_col, &units)
}

/// Lay out a fact / relation cell `NAME( a1, a2, … )`. If its flat rendering fits
/// [`FILL_WIDTH`] it stays on one line. Otherwise its arguments fill at the
/// argument column; a sole tuple argument recurses into [`layout_tuple`]; and the
/// closing `)` peels onto its own line at column 0 (the padded ` )` space becomes
/// the break).
fn layout_fact(flat: &str) -> Vec<PLine> {
    if width(flat) <= FILL_WIDTH {
        return vec![(0, flat.to_string())];
    }
    let Some(lp) = flat.find("( ") else {
        return vec![(0, flat.to_string())];
    };
    if !flat.ends_with(" )") {
        return vec![(0, flat.to_string())];
    }
    let open_str = &flat[..lp + 2]; // "NAME( "
    let content = &flat[lp + 2..flat.len() - 2];
    let open_col = width(open_str);
    let args = split_top_commas(content);

    // A sole tuple argument: the fact opens "NAME( <" and the tuple fills inside.
    if args.len() == 1 && is_tuple(&args[0]) {
        let inner = &args[0][1..args[0].len() - 1];
        let tlines = layout_tuple(inner, open_col);
        let mut lines: Vec<PLine> = Vec::with_capacity(tlines.len() + 2);
        lines.push((0, format!("{}{}", open_str, tlines[0].1)));
        for t in &tlines[1..] {
            lines.push(t.clone());
        }
        lines.push((0, ")".to_string()));
        return lines;
    }

    // General fill of the (atomic) arguments; the fact's `)` always peels to col 0.
    let mut units: Vec<Unit> = Vec::with_capacity(args.len() + 1);
    for (i, a) in args.iter().enumerate() {
        units.push(Unit { sep: if i == 0 { "" } else { ", " }, text: a.clone(), indent: open_col, hard: false });
    }
    units.push(Unit { sep: "", text: ")".to_string(), indent: 0, hard: true });
    run_layout(open_str, open_col, &units)
}

/// Lay out an info cell `#t : Rule[a1, a2, …]`. The action list uses a vertical
/// `sep` — one action per line when the cell overflows [`FILL_WIDTH`] — with the
/// trailing `,` kept on each non-final line and the `]` attached to the last
/// action.
fn layout_info(flat: &str) -> Vec<PLine> {
    if width(flat) <= FILL_WIDTH {
        return vec![(0, flat.to_string())];
    }
    let Some(lb) = flat.find('[') else {
        return vec![(0, flat.to_string())];
    };
    if !flat.ends_with(']') {
        return vec![(0, flat.to_string())];
    }
    let open_str = &flat[..lb + 1]; // "#t : Rule["
    let content = &flat[lb + 1..flat.len() - 1];
    let open_col = width(open_str);
    let acts = split_top_commas(content);
    let mut units: Vec<Unit> = Vec::with_capacity(acts.len());
    for (i, a) in acts.iter().enumerate() {
        // Each action begins its own line (a vertical `sep`, hard break after the
        // first); non-final actions carry a trailing `,`, the last carries `]`.
        let text = if i + 1 < acts.len() { format!("{},", a) } else { format!("{}]", a) };
        units.push(Unit { sep: "", text, indent: open_col, hard: i > 0 });
    }
    run_layout(open_str, open_col, &units)
}

/// Render one record cell from its flat (un-escaped) text into the exact label
/// bytes: single line (escaped) when it fits [`FILL_WIDTH`], otherwise the wrapped
/// form — physical lines each prefixed by their `&nbsp;` indent, escaped, and
/// terminated by `\l` (trailing `\l` included), per BEHAVIOR.md §3f.
///
/// Dispatch by cell shape: an info cell (`#t : Rule[…]`) uses the vertical action
/// `sep`; a fact / relation cell (`NAME( … )`) uses the greedy argument/tuple fill
/// with the delimiter peel. The same entry serves the Term-based path (feed
/// [`Fact::render_flat`]) and the pre-rendered-string path (feed the caller's
/// already-rendered cell text): the wrap and escape apply identically.
pub fn wrap_cell(flat: &str) -> String {
    let lines = if flat.starts_with('#') && flat.contains('[') {
        layout_info(flat)
    } else {
        layout_fact(flat)
    };
    if lines.len() == 1 && lines[0].0 == 0 {
        return escape_record(&lines[0].1);
    }
    let mut out = String::new();
    for (indent, text) in &lines {
        out.push_str(&"&nbsp;".repeat(*indent));
        out.push_str(&escape_record(text));
        out.push_str("\\l");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_record_metachars() {
        assert_eq!(escape_record("<$A, $A, 'g'^~ex>"), "\\<$A, $A, 'g'^~ex\\>");
        assert_eq!(escape_record("a|b{c}"), "a\\|b\\{c\\}");
        assert_eq!(escape_record("no meta 'g'^~x"), "no meta 'g'^~x");
    }

    #[test]
    fn fact_flat_spacing_matches_corpus() {
        // Fr( ~ex )  — a fact pads inside its parens.
        assert_eq!(Fact::new("Fr", vec![Term::fresh("ex")]).render_flat(), "Fr( ~ex )");
        // zero-arg fact: NoA( )
        assert_eq!(Fact::new("NoA", vec![]).render_flat(), "NoA( )");
        // Out( pk(~ltkA) ) — outer fact pads; inner function pk(...) does NOT.
        let f = Fact::new("Out", vec![Term::app("pk", vec![Term::fresh("ltkA")])]);
        assert_eq!(f.render_flat(), "Out( pk(~ltkA) )");
        // !SessionKey( <$A, $A, 'g'^~ex>, $A, ~ex )
        let f = Fact::new(
            "!SessionKey",
            vec![
                Term::tuple(vec![Term::pubv("A"), Term::pubv("A"), Term::exp(Term::cst("g"), Term::fresh("ex"))]),
                Term::pubv("A"),
                Term::fresh("ex"),
            ],
        );
        assert_eq!(f.render_flat(), "!SessionKey( <$A, $A, 'g'^~ex>, $A, ~ex )");
    }

    #[test]
    fn info_cell_shape() {
        assert_eq!(render_info("vf.5", "Fresh", &[]), "#vf.5 : Fresh");
        let acts = vec![Fact::new("RegKey", vec![Term::pubv("R.4")])];
        assert_eq!(render_info("vr.9", "generate_ltk", &acts), "#vr.9 : generate_ltk[RegKey( $R.4 )]");
    }

    #[test]
    fn wrap_format_reproduces_observed_action_cell() {
        // Reproduce the observed record cell verbatim (from corpus fixture
        // 004825b0f93b1a8c.dot, node n24's info cell): the action list of
        // Corrupt_SessionKey wraps, aligning the continuation under the `[`.
        //   #vr.2 : Corrupt_SessionKey[Corrupt( $A ),\l<27 nbsp>BeforeExpire( <…> )]\l
        // The info prefix `#vr.2 : Corrupt_SessionKey[` is exactly 27 columns, so
        // the continuation indent is 27 `&nbsp;`.
        let prefix = "#vr.2 : Corrupt_SessionKey[";
        assert_eq!(prefix.chars().count(), 27);
        let a1 = "Corrupt( $A ),";
        let a2 = escape_record("BeforeExpire( <$A, $A, 'g'^~ex> )]");
        let body = join_wrapped(&[a1.to_string(), a2], 27);
        // The exact bytes that follow `<n22> #vr.2 : Corrupt_SessionKey[` in the
        // corpus cell, including the trailing `\l`.
        let observed_cell_tail = "Corrupt( $A ),\\l\
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;\
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;\
BeforeExpire( \\<$A, $A, 'g'^~ex\\> )]\\l";
        assert_eq!(body, observed_cell_tail);
    }

    #[test]
    fn fill_packs_greedily_to_width() {
        // Two short items that fit share a line; a third that overflows wraps.
        let items = vec!["aa".to_string(), "bb".to_string(), "cccccc".to_string()];
        let lines = fill(&items, ", ", 4, 12);
        // col starts at 4: "aa"(6) + ", bb"(10) fits <=12; ", cccccc" -> 18 > 12, wrap.
        assert_eq!(lines, vec!["aa, bb", "cccccc"]);
    }

    // ---- Round-4: the record-cell wrap DECISION (BEHAVIOR.md §3f) -----------
    // All expectations below are OBSERVED live-server output from crafted
    // single-node theories `Out(<'a01', …>)` / `Out(<'aaa…', 'y'>)`, captured by
    // a one-column width sweep (workspace/QUERIES.log, Session 4).

    #[test]
    fn flat_fit_boundary_is_87_columns() {
        // A fact rendered to exactly 87 columns stays on one line; 88 wraps.
        // Observed: `Out( <'aa…(71 a)', 'y'> )` = 87 cols -> single line;
        //           `Out( <'aa…(72 a)', 'y'> )` = 88 cols -> wraps.
        let at87: String = format!("Out( <'{}', 'y'> )", "a".repeat(71));
        let at88: String = format!("Out( <'{}', 'y'> )", "a".repeat(72));
        assert_eq!(at87.chars().count(), 87);
        assert_eq!(at88.chars().count(), 88);
        assert!(fits_one_line(&at87));
        assert!(!fits_one_line(&at88));
        assert_eq!(FILL_WIDTH, 87);
    }

    #[test]
    fn flat_fit_matches_captured_flat_fixtures() {
        // Two captured single-line cells whose flat width is <= 87.
        // `Out( <'a01', …, 'a11'> )` = 84 cols (observed single-line).
        let elems: Vec<String> = (1..=11).map(|k| format!("'a{:02}'", k)).collect();
        let nelem11 = format!("Out( <{}> )", elems.join(", "));
        assert_eq!(nelem11.chars().count(), 84);
        assert!(fits_one_line(&nelem11));
    }

    // ---- Round-5: wrap_cell byte-exact wrap + delimiter peel (BEHAVIOR.md §3f) --
    // Expectations are the exact record-cell tails observed live (wrap_E*/W* fixtures).

    #[test]
    fn wrap_cell_single_line_when_fits() {
        // <= 87 columns: escaped flat, no `\l`.
        assert_eq!(wrap_cell("Fr( ~s )"), "Fr( ~s )");
        assert_eq!(
            wrap_cell("Out( <$R, $I, 'g'^~ekR> )"),
            "Out( \\<$R, $I, 'g'^~ekR\\> )"
        );
    }

    #[test]
    fn wrap_cell_tuple_fill_and_paren_peel() {
        // Out(<'a01'..'a12'>) (flat 91): line0 packs 11 elems + trailing ", ",
        // 'a12' wraps to col 6, tuple '>' stays, fact ')' peels to col 0.
        let elems: Vec<String> = (1..=12).map(|k| format!("'a{:02}'", k)).collect();
        let flat = format!("Out( <{}> )", elems.join(", "));
        let want = "Out( \\<'a01', 'a02', 'a03', 'a04', 'a05', 'a06', 'a07', 'a08', 'a09', 'a10', 'a11', \\l\
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;'a12'\\>\\l)\\l";
        assert_eq!(wrap_cell(&flat), want);
    }

    #[test]
    fn wrap_cell_tuple_close_peels_to_open_column() {
        // Out(<'a…a'(74), 'y'>) (flat 90): last element fills col 87 so '>' peels to
        // the '<' column (5), then ')' peels to col 0.
        let flat = format!("Out( <'{}', 'y'> )", "a".repeat(74));
        let want = format!(
            "Out( \\<'{}', 'y'\\l&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;\\>\\l)\\l",
            "a".repeat(74)
        );
        assert_eq!(wrap_cell(&flat), want);
    }

    #[test]
    fn wrap_cell_info_action_list_is_vertical() {
        // Info cells over 87 columns place one action per line (vertical `sep`),
        // the `]` attached to the last action, continuation indent = prefix length.
        let prefix = "#vr.7 : SomeLongRuleName";
        let a1 = "FirstActionFact( $A, $B, $C, $D )";
        let a2 = "SecondActionFact( $A, $B, $C, $D )";
        let flat = format!("{}[{}, {}]", prefix, a1, a2);
        assert!(flat.chars().count() > FILL_WIDTH);
        let open_col = prefix.chars().count() + 1; // after '['
        let want = format!(
            "{}[{},\\l{}{}]\\l",
            prefix,
            a1,
            "&nbsp;".repeat(open_col),
            a2
        );
        assert_eq!(wrap_cell(&flat), want);
    }

    #[test]
    fn paragraph_fill_reproduces_observed_first_line() {
        // Observed `Out(<'a01'..'a12'>)`: the tuple overflows (flat 91 > 87) and
        // line 0 holds eleven elements PLUS the trailing ", " (col 6..83), the
        // twelfth wrapping. The tuple opens at column 6 (after `Out( <`).
        let elems: Vec<String> = (1..=12).map(|k| format!("'a{:02}'", k)).collect();
        let lines = paragraph_fill(&elems, ", ", 6);
        // First physical line = eleven elements joined by ", " with a trailing ", ".
        let expect_line0: String = {
            let first11: Vec<String> = (1..=11).map(|k| format!("'a{:02}'", k)).collect();
            format!("{}, ", first11.join(", "))
        };
        assert_eq!(lines[0], expect_line0);
        // Its column extent from the tuple-open column matches the observed 83.
        assert_eq!(6 + lines[0].chars().count(), 83);
        // The overflowing twelfth element starts the next physical line.
        assert_eq!(lines[1], "'a12'");
    }
}

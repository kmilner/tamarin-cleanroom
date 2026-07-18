//! Reconstructing how a record cell's flat term text is fed through the layout
//! engine ([`crate::pretty`]).
//!
//! The engine ([`crate::pretty`]) is the reference toolchain's BSD pretty-printer,
//! ported faithfully. This module carries the tamarin-SPECIFIC part: which
//! combinators wrap a fact / tuple / info cell, with what padding, and at what
//! line width — every one of which was pinned by black-box probes (BEHAVIOR.md
//! §3f; QUERIES.log wrap sweeps E10..E14 / W69..W74 and the Wide-rule case graph).
//!
//! Reconstructed cell grammar (each byte-verified against captured fixtures):
//!   * **fact / relation cell** `Name( a, b, … )` — the arguments are laid out as
//!     a `fsep` (paragraph fill, comma-punctuated) that opens after `Name( ` and
//!     self-aligns its continuation lines to that column; the whole thing is an
//!     outer `sep [ Name( args , ) ]` so the closing `)` drops to column 0 on its
//!     own line exactly when the argument fill wrapped (the ` )` padding space is
//!     the break). Zero-argument facts render `Name( )` flat.
//!   * **tuple** `<e1, …, en>` — `'<' <> fcat(e1, ", ", …, en, nest(-1) '>')`: the
//!     elements fill (paragraph, `, `-punctuated) and self-align one column past
//!     the `<`; the closing `>` is a fill element nested back one column, so it
//!     stays beside the last element when it fits and otherwise peels onto its own
//!     line aligned under the `<`.
//!   * **info cell** `#t : Rule[a1, …]` — the action list is a `vcat` (one action
//!     per line) wrapped in `[ … ]`; with two or more actions it is therefore
//!     always vertical, and a lone action fills only when the cell overflows.
//!
//! The fit **budget** a cell is laid out at is the HughesPJ *ribbon*; for a lone
//! cell it is [`FILL_WIDTH`] (= 87, the probed flat-fit boundary). The line length
//! is `1.5 ×` the ribbon ([`RIBBONS`] = 1.5, the HughesPJ default), and that gap
//! is what makes `fill` produce *ragged* paragraph fills — a physical line may be
//! shorter than a later one (e.g. two arguments then four wider ones) — which a
//! greedy fill can never do and which the reference's output requires (see
//! [`budget_line_len`] and the `ragged_fill_line0_shorter_than_line1` test). How
//! the cells of one record group share the row is a separate, probe-derived
//! two-layer (wrap trigger + fill share) allocation and lives in
//! [`crate::generate::group_widths`].

use crate::pretty::{beside_op, char as pchar, fcat, fsep, nest, render_page, sep, text, vcat, Doc};

/// The fit **budget** (ribbon) the reference lays a lone record cell out at
/// (BEHAVIOR.md §3f): a fact whose flat rendering is `≤ 87` columns stays on one
/// line, `88` breaks. A record group shares this budget among its cells
/// (`crate::generate`); this is the per-cell budget of a *lone* cell.
pub const FILL_WIDTH: isize = 87;

/// Ribbons-per-line the reference uses for cell layout (HughesPJ default). The
/// probed flat-fit boundary is the **ribbon** (content width excluding
/// indentation), and the line length is `1.5 ×` the ribbon — a gap that is what
/// makes HughesPJ `fill` produce *ragged* paragraph fills (a physical line may be
/// shorter than a later one, e.g. `2` args then `4` wider args), the reference's
/// observed behavior. A cell's budget `B` is its ribbon; it is rendered at line
/// length [`budget_line_len`]`(B)` so `round(lineLen / 1.5) == B`.
pub const RIBBONS: f64 = 1.5;

/// The HughesPJ line length that yields ribbon (= fit budget) `budget` at
/// [`RIBBONS`] = 1.5: `lineLen = ⌊3·budget / 2⌋`, so `round(lineLen / 1.5) ==
/// budget`. Content still fits in `budget` columns, but the wider line length
/// gives `fill` room to look ahead and produce ragged fills.
pub fn budget_line_len(budget: isize) -> isize {
    3 * budget / 2
}

/// Split `s` at top-level `", "` separators, honoring `()`/`<>`/`[]` nesting and
/// `'…'` quotes. Returns the comma-separated pieces (each un-trimmed).
pub fn split_top_commas(s: &str) -> Vec<String> {
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

/// Count top-level actions in an info action list `#t : Rule[ … ]` (0 if none).
pub fn count_info_actions(flat: &str) -> usize {
    let Some(open) = flat.find('[') else { return 0 };
    let inner = &flat[open + 1..];
    let inner = inner.strip_suffix(']').unwrap_or(inner);
    if inner.is_empty() {
        return 0;
    }
    split_top_commas(inner).len()
}

fn is_tuple(s: &str) -> bool {
    s.starts_with('<') && s.ends_with('>') && s.len() >= 2
}

/// Split `s` at top-level `++` separators (a `(a++b++…)` union body), honoring
/// nesting and quotes. Returns one piece when there is no top-level `++`.
pub fn split_top_unions(s: &str) -> Vec<String> {
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
            '+' if depth == 0 && i + 1 < chars.len() && chars[i + 1] == '+' => {
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

/// A parenthesized `++`-union `(a++b++…)`: ≥ 2 top-level `++`-separated pieces.
fn union_parts(s: &str) -> Option<Vec<String>> {
    if !(s.starts_with('(') && s.ends_with(')')) || s.len() < 2 {
        return None;
    }
    let parts = split_top_unions(&s[1..s.len() - 1]);
    if parts.len() >= 2 { Some(parts) } else { None }
}

/// An unpadded function application `name(args)`: identifier directly followed
/// by `(` (no space after) with the matching `)` closing the string. Returns
/// `(name, args)`.
fn func_parts(s: &str) -> Option<(&str, Vec<String>)> {
    let open = s.find('(')?;
    if open == 0 || !s.ends_with(')') || s.len() < open + 2 {
        return None;
    }
    let name = &s[..open];
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '!') {
        return None;
    }
    let inner = &s[open + 1..s.len() - 1];
    if inner.starts_with(' ') {
        return None; // padded fact form, not a function application
    }
    // the closing paren must match the opening one (reject `f(x)^g(y)`)
    let mut depth = 0i32;
    let mut in_quote = false;
    for (idx, c) in s.char_indices() {
        if in_quote {
            if c == '\'' {
                in_quote = false;
            }
            continue;
        }
        match c {
            '\'' => in_quote = true,
            '(' | '<' | '[' => depth += 1,
            ')' | '>' | ']' => {
                depth -= 1;
                if depth == 0 && idx != s.len() - 1 {
                    return None;
                }
            }
            _ => {}
        }
    }
    Some((name, split_top_commas(inner)))
}

/// A `Doc` for one tuple element / fact argument. Tuples, parenthesized
/// `++`-unions and function applications recurse (each breaks internally in
/// the observed cells — round-10 batteries B/D); every other shape (atom,
/// exponentiation, other AC operator term) is a single fill token rendered
/// from its flat text.
fn arg_doc(s: &str) -> Doc {
    if is_tuple(s) {
        tuple_doc(&s[1..s.len() - 1])
    } else if let Some(parts) = union_parts(s) {
        union_doc(&parts)
    } else if let Some((name, args)) = func_parts(s) {
        func_doc(name, &args)
    } else {
        text(s)
    }
}

/// `(a++b++…)` — the elements fill and align one past the `(`; each `++` stays
/// attached to the element it follows; the closing `)` is a fill element
/// nested back one column, so it stays beside the last element when it fits
/// and peels onto its own line under the `(` otherwise (probe battery R10-D:
/// UA_20 fill + UB_39 `)`-peel vs UB_40 attached).
fn union_doc(parts: &[String]) -> Doc {
    let n = parts.len();
    let mut toks: Vec<Doc> = Vec::with_capacity(n + 1);
    for (i, e) in parts.iter().enumerate() {
        let d = arg_doc(e);
        if i + 1 < n {
            toks.push(beside_op(d, text("++")));
        } else {
            toks.push(d);
        }
    }
    toks.push(nest(-1, &pchar(')')));
    beside_op(pchar('('), fcat(toks))
}

/// `name(a, b, …)` — an unpadded function application: the arguments fill
/// after `name(` (continuations align at that column, one column deeper per
/// nesting level in a chain), and the closing `)` stays ATTACHED to the last
/// argument (probe battery R10-B: FD_90 internal break, FC_1/FC_3 fills and
/// chains — the `)` never peels alone, unlike the fact-level `)`).
fn func_doc(name: &str, args: &[String]) -> Doc {
    let n = args.len();
    let toks: Vec<Doc> = args
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let d = arg_doc(a);
            if i + 1 < n {
                beside_op(d, text(","))
            } else {
                beside_op(d, pchar(')'))
            }
        })
        .collect();
    beside_op(text(&format!("{}(", name)), fsep(toks))
}

/// `<e1, …, en>` — the elements fill and align one past the `<`; the `>` is a
/// fill element nested back one column so it stays beside the last element when
/// it fits and peels under the `<` otherwise.
fn tuple_doc(inner: &str) -> Doc {
    let elems = split_top_commas(inner);
    let n = elems.len();
    let mut toks: Vec<Doc> = Vec::with_capacity(n + 1);
    for (i, e) in elems.iter().enumerate() {
        let d = arg_doc(e);
        if i + 1 < n {
            toks.push(beside_op(d, text(", ")));
        } else {
            toks.push(d);
        }
    }
    toks.push(nest(-1, &pchar('>')));
    beside_op(pchar('<'), fcat(toks))
}

/// `Name( a, b, … )` — the argument list is a comma-punctuated `fsep` opening
/// after `Name( `; the outer `sep [ …, ) ]` drops the closing `)` to its own line
/// exactly when the argument fill wraps.
fn fact_doc(name: &str, args: &[String]) -> Doc {
    if args.is_empty() {
        return text(&format!("{}( )", name));
    }
    let n = args.len();
    let toks: Vec<Doc> = args
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let d = arg_doc(a);
            if i + 1 < n {
                beside_op(d, text(","))
            } else {
                d
            }
        })
        .collect();
    let args_doc = fsep(toks);
    let opened = beside_op(text(&format!("{}( ", name)), args_doc);
    sep(vec![opened, text(")")])
}

/// `#t : Rule[a1, …]` — the action list is a `vcat` (one action per line) inside
/// `[ … ]`; `#t : Rule` with no actions is bare text.
fn info_doc(flat: &str) -> Doc {
    let Some(open) = flat.find('[') else {
        return text(flat);
    };
    if !flat.ends_with(']') {
        return text(flat);
    }
    let prefix = &flat[..open + 1]; // "#t : Rule["
    let inner = &flat[open + 1..flat.len() - 1];
    let actions = split_top_commas(inner);
    let n = actions.len();
    let action_docs: Vec<Doc> = actions
        .iter()
        .enumerate()
        .map(|(i, a)| {
            // Each action is a fact that may itself fill-wrap; non-final actions
            // carry a trailing comma.
            let d = fact_from_flat(a);
            if i + 1 < n {
                beside_op(d, text(","))
            } else {
                d
            }
        })
        .collect();
    beside_op(beside_op(text(prefix), vcat(action_docs)), text("]"))
}

/// Build a fact `Doc` from a flat fact string `Name( … )` (or a bare atom).
fn fact_from_flat(flat: &str) -> Doc {
    if let Some((name, args)) = parse_fact(flat) {
        fact_doc(name, &args)
    } else {
        text(flat)
    }
}

/// Parse a padded fact `Name( a, b )` / `Name( )` into `(name, args)`. Returns
/// `None` for anything that is not a padded fact (an atom, a function `f(x)`
/// without inner padding, etc.).
fn parse_fact(flat: &str) -> Option<(&str, Vec<String>)> {
    if flat == "" {
        return None;
    }
    // Zero-arg: "Name( )"
    if let Some(name) = flat.strip_suffix("( )") {
        if !name.is_empty() && !name.contains(['(', ')', '<', '>', ' ']) {
            return Some((name, Vec::new()));
        }
    }
    let open = flat.find("( ")?;
    if !flat.ends_with(" )") {
        return None;
    }
    let name = &flat[..open];
    // The name must be a plain relation/fact symbol (optionally `!`-prefixed).
    if name.is_empty()
        || name.contains(['(', ')', '<', '>', ' ', ','])
    {
        return None;
    }
    let content = &flat[open + 2..flat.len() - 2];
    Some((name, split_top_commas(content)))
}

/// Build a `Doc` for one record cell from its flat (post-abbreviation,
/// un-escaped) text, dispatching by cell shape (info / fact / atom).
pub fn cell_doc(flat: &str) -> Doc {
    if flat.starts_with('#') && flat.contains('[') {
        info_doc(flat)
    } else if let Some((name, args)) = parse_fact(flat) {
        fact_doc(name, &args)
    } else {
        text(flat)
    }
}

/// Render a cell's flat text to physical lines `(indent, content)` at the given
/// fit **budget** (ribbon), via the layout engine. `indent` is the count of
/// leading spaces the engine emitted (→ `&nbsp;` in the record label); `content`
/// is the line with that indentation stripped, still un-escaped. The cell fits on
/// one line iff its flat width ≤ `budget`; the line length is
/// [`budget_line_len`]`(budget)` (= 1.5·budget) so wrapping is ragged (§3f).
pub fn layout_lines(flat: &str, budget: isize) -> Vec<(usize, String)> {
    layout_lines_lr(flat, budget_line_len(budget), RIBBONS)
}

/// Like [`layout_lines`] but with an explicit `line_len` and `ribbons_per_line`
/// (ribbon = round(line_len / ribbons_per_line)). When `line_len` exceeds the
/// ribbon, HughesPJ `fill` produces ragged paragraph fills (a physical line may
/// be shorter than a later one), the reference's observed behavior.
pub fn layout_lines_lr(flat: &str, line_len: isize, ribbons_per_line: f64) -> Vec<(usize, String)> {
    let doc = cell_doc(flat);
    let rendered = render_page(line_len, ribbons_per_line, &doc);
    rendered
        .split('\n')
        .map(|line| {
            let indent = line.len() - line.trim_start_matches(' ').len();
            (indent, line[indent..].to_string())
        })
        .collect()
}

/// Like [`wrap_cell_dot`] but with an explicit line length and ribbons-per-line.
pub fn wrap_cell_dot_lr(flat: &str, line_len: isize, ribbons_per_line: f64) -> String {
    let lines = layout_lines_lr(flat, line_len, ribbons_per_line);
    if lines.len() == 1 && lines[0].0 == 0 {
        return crate::render::escape_record(&lines[0].1);
    }
    let mut out = String::new();
    for (indent, content) in &lines {
        out.push_str(&"&nbsp;".repeat(*indent));
        out.push_str(&crate::render::escape_record(content));
        out.push_str("\\l");
    }
    out
}

/// Render a cell's flat text into the exact graphviz record-label bytes at the
/// given fit **budget** (ribbon): a single escaped line when the flat width ≤
/// `budget`, otherwise each physical line prefixed by its `&nbsp;` indent,
/// escaped, and terminated by `\l` (trailing `\l` included), per BEHAVIOR.md §3f.
/// The layout is computed on the UN-escaped text (so `<`/`>` count one column
/// each, matching the reference), and escaping is applied afterwards. Wrapping is
/// ragged (line length = 1.5·budget; see [`RIBBONS`]).
pub fn wrap_cell_dot(flat: &str, budget: isize) -> String {
    let lines = layout_lines(flat, budget);
    if lines.len() == 1 && lines[0].0 == 0 {
        return crate::render::escape_record(&lines[0].1);
    }
    let mut out = String::new();
    for (indent, content) in &lines {
        out.push_str(&"&nbsp;".repeat(*indent));
        out.push_str(&crate::render::escape_record(content));
        out.push_str("\\l");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convert engine lines to the DOT record-label byte format (no escaping here;
    /// tests compare the raw text form to check structure).
    fn to_record(flat: &str, width: isize) -> String {
        let lines = layout_lines(flat, width);
        if lines.len() == 1 && lines[0].0 == 0 {
            return lines[0].1.clone();
        }
        let mut out = String::new();
        for (indent, content) in &lines {
            out.push_str(&"&nbsp;".repeat(*indent));
            out.push_str(content);
            out.push_str("\\l");
        }
        out
    }

    #[test]
    fn fact_single_line_fits() {
        assert_eq!(to_record("Fr( ~s )", 87), "Fr( ~s )");
        assert_eq!(to_record("Out( <$R, $I, 'g'^~ekR> )", 87), "Out( <$R, $I, 'g'^~ekR> )");
    }

    #[test]
    fn e12_tuple_fill_and_paren_peel() {
        let elems: Vec<String> = (1..=12).map(|k| format!("'a{:02}'", k)).collect();
        let flat = format!("Out( <{}> )", elems.join(", "));
        let want = "Out( <'a01', 'a02', 'a03', 'a04', 'a05', 'a06', 'a07', 'a08', 'a09', 'a10', 'a11', \\l\
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;'a12'>\\l)\\l";
        assert_eq!(to_record(&flat, 87), want);
    }

    #[test]
    fn w74_tuple_close_peels_to_open_column() {
        let flat = format!("Out( <'{}', 'y'> )", "a".repeat(74));
        let want = format!(
            "Out( <'{}', 'y'\\l&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;>\\l)\\l",
            "a".repeat(74)
        );
        assert_eq!(to_record(&flat, 87), want);
    }

    #[test]
    fn ragged_fill_line0_shorter_than_line1() {
        // The RAGGED fill (ribbonsPerLine = 1.5): a fact whose arguments wrap with
        // a physical line0 SHORTER than a later line — impossible for a greedy
        // fill. Corpus cell (`St_1_gNB`, e.g. `00664cc78ede5046.dot`): line0 holds
        // 2 args, line1 holds 4 WIDER args aligned under the `(`, then `)`. The
        // budget (25) is the cell's proportional share of its group's 87.
        let flat = "St_1_gNB( ~gNB_ID, KD8, KD1, '0', AM2, GN1 )";
        let want = "St_1_gNB( ~gNB_ID, KD8,\\l\
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;KD1, '0', AM2, GN1\\l)\\l";
        assert_eq!(to_record(flat, 25), want);
        // The continuation aligns to the `(` column (10 = width of "St_1_gNB( ").
        assert_eq!("St_1_gNB( ".chars().count(), 10);
    }

    #[test]
    fn ack_multi_arg_fact_break() {
        // Wide-rule Ack conclusion at its group width (20): fact-arg break at the
        // bare comma, tuple stays, ')' peels to col 0.
        let flat = "Ack( ~n.4, <x1.4, x2.4> )";
        let want = "Ack( ~n.4,\\l&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<x1.4, x2.4>\\l)\\l";
        assert_eq!(to_record(flat, 20), want);
        // Lone (width 87) stays flat.
        assert_eq!(to_record(flat, 87), "Ack( ~n.4, <x1.4, x2.4> )");
    }

    #[test]
    fn func_internal_break_and_chain() {
        // Live battery R10-B: a lone `Q( wwwww(16 args) )` at flat 90 breaks
        // INSIDE the function — args fill to 87, continuation aligned after
        // `wwwww(`, the func `)` ATTACHED to the last arg, fact `)` peeled
        // (probeB_dots/l_FD_90.dot, byte-exact).
        let args: Vec<String> = (0..16)
            .map(|k| format!("$a{}", (b'a' + k as u8) as char))
            .collect();
        let flat = format!("Q( wwwww({}) )", args.join(", "));
        let want = "Q( wwwww($aa, $ab, $ac, $ad, $ae, $af, $ag, $ah, $ai, $aj, $ak, $al, $am, $an, $ao,\\l\
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;$ap)\\l)\\l";
        assert_eq!(to_record(&flat, 87), want);
        // A flat-88 func fact wraps by fact-paren peel only (l_FD_88.dot).
        let flat88 = format!("Q( www({}) )", args.join(", "));
        let want88 = format!("Q( www({})\\l)\\l", args.join(", "));
        assert_eq!(to_record(&flat88, 87), want88);
        // Deep right-nested chain: one break per level, indent +2 per level,
        // the tail rides flat once it fits (l_FC_3.dot, byte-exact).
        let mut chain = "$an".to_string();
        for i in (0..13).rev() {
            chain = format!("p($a{}, {})", (b'a' + i as u8) as char, chain);
        }
        let got = to_record(&format!("Q( {} )", chain), 87);
        let want_chain = "Q( p($aa,\\l\
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;p($ab,\\l\
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;p($ac,\\l\
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;p($ad, p($ae, p($af, p($ag, p($ah, p($ai, p($aj, p($ak, p($al, p($am, $an)))))))))))))\\l)\\l";
        assert_eq!(got, want_chain);
    }

    #[test]
    fn union_fill_and_close_peel() {
        // Live battery R10-D: unions display parenthesized/unspaced, break
        // AFTER `++` with the continuation one past the `(` (l_UA_20.dot).
        let elems: Vec<String> = (0..20)
            .map(|k| format!("$a{}", (b'a' + k as u8) as char))
            .collect();
        let flat = format!("U( ({}) )", elems.join("++"));
        let want = "U( ($aa++$ab++$ac++$ad++$ae++$af++$ag++$ah++$ai++$aj++$ak++$al++$am++$an++$ao++$ap++\\l\
&nbsp;&nbsp;&nbsp;&nbsp;$aq++$ar++$as++$at)\\l)\\l";
        assert_eq!(to_record(&flat, 87), want);
        // The union `)` peels to the `(` column when it does not fit beside
        // the last element (l_UB_39.dot: line0 ends at 87 exactly) …
        let q39 = format!("'{}05'", "z".repeat(39));
        let flat39 = format!("U( ({}++$aa++$ab++$ac++$ad++$ae++$af++$ag++$ah) )", q39);
        let want39 = format!(
            "U( ({}++$aa++$ab++$ac++$ad++$ae++$af++$ag++$ah\\l&nbsp;&nbsp;&nbsp;)\\l)\\l",
            q39
        );
        assert_eq!(to_record(&flat39, 87), want39);
        // … and stays attached when the last element wraps with it
        // (l_UB_40.dot).
        let q40 = format!("'{}06'", "z".repeat(40));
        let flat40 = format!("U( ({}++$aa++$ab++$ac++$ad++$ae++$af++$ag++$ah) )", q40);
        let want40 = format!(
            "U( ({}++$aa++$ab++$ac++$ad++$ae++$af++$ag++\\l&nbsp;&nbsp;&nbsp;&nbsp;$ah)\\l)\\l",
            q40
        );
        assert_eq!(to_record(&flat40, 87), want40);
    }

    #[test]
    fn info_two_actions_vertical() {
        let flat = "#i : R[A( x ), B( y )]";
        let want = "#i : R[A( x ),\\l&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;B( y )]\\l";
        assert_eq!(to_record(flat, 87), want);
        // One short action stays on one line.
        assert_eq!(to_record("#i : R[A( x )]", 87), "#i : R[A( x )]");
    }
}

#[cfg(test)]
mod explore {
    use super::*;
    #[test]
    #[ignore]
    fn big_element_counts() {
        let elems: Vec<String> = (1..=10).map(|k| format!("x{}.4", k)).collect();
        let big = format!("Big( <{}> )", elems.join(", "));
        let ins = format!("In( <{}> )", elems.join(", "));
        println!("Big flat = {}", big.chars().count());
        println!("In  flat = {}", ins.chars().count());
        for w in [48,51,53,54,55,56,57,58,67,77] {
            let lines = layout_lines(&big, w);
            let l0 = &lines[0].1;
            let cnt = l0.matches(".4").count();
            println!("Big@{w:2} -> {} lines, line0 elems={} : {:?}", lines.len(), cnt, l0);
        }
        for w in [66,67,68,77,87] {
            let lines = layout_lines(&ins, w);
            println!("In @{w:2} -> {} lines : {:?}", lines.len(), lines[0].1);
        }
    }
}

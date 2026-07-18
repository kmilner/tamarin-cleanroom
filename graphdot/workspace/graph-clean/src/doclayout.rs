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
//! The line WIDTH a cell is laid out at is [`crate::pretty::render_page`]'s line
//! length; for a lone cell it is [`FILL_WIDTH`] (= 87, the probed flat-fit
//! boundary). How the cells of one record group share that width is a separate,
//! probe-derived allocation and lives in [`crate::generate`].

use crate::pretty::{beside_op, char as pchar, fcat, fsep, nest, render_page, sep, text, vcat, Doc};

/// Line length the reference lays a lone record cell out at (BEHAVIOR.md §3f): a
/// fact whose flat rendering is `≤ 87` columns stays on one line, `88` breaks.
pub const FILL_WIDTH: isize = 87;

/// Ribbons-per-line the reference uses for cell layout. The probed boundary is a
/// single absolute column budget measured from column 0, i.e. `min(lineLength,
/// ribbon) == lineLength`, so ribbon does not bind — modelled as `1.0`.
pub const RIBBONS: f64 = 1.0;

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

/// A `Doc` for one tuple element / fact argument. Tuples recurse; every other
/// shape (atom, function application, exponentiation, AC term) is a single fill
/// token rendered from its flat text (these do not re-wrap internally in the
/// observed cells — the wrap happens at the enclosing fact/tuple level).
fn arg_doc(s: &str) -> Doc {
    if is_tuple(s) {
        tuple_doc(&s[1..s.len() - 1])
    } else {
        text(s)
    }
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
/// line width, via the layout engine. `indent` is the count of leading spaces the
/// engine emitted (→ `&nbsp;` in the record label); `content` is the line with
/// that indentation stripped, still un-escaped.
pub fn layout_lines(flat: &str, width: isize) -> Vec<(usize, String)> {
    let doc = cell_doc(flat);
    let rendered = render_page(width, RIBBONS, &doc);
    rendered
        .split('\n')
        .map(|line| {
            let indent = line.len() - line.trim_start_matches(' ').len();
            (indent, line[indent..].to_string())
        })
        .collect()
}

/// Render a cell's flat text into the exact graphviz record-label bytes at the
/// given line width: a single escaped line when it fits, otherwise each physical
/// line prefixed by its `&nbsp;` indent, escaped, and terminated by `\l`
/// (trailing `\l` included), per BEHAVIOR.md §3f. The layout is computed on the
/// UN-escaped text (so `<`/`>` count one column each, matching the reference),
/// and escaping is applied afterwards.
pub fn wrap_cell_dot(flat: &str, width: isize) -> String {
    let lines = layout_lines(flat, width);
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

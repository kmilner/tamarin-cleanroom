//! Corpus constraint extractor (round 9): for every prem/concl record group in
//! the corpus, compute per cell the SET of line lengths (engine L-space,
//! ribbonsPerLine = 1.5) at which the exact layout engine reproduces the
//! observed record-cell bytes. Emits one TSV line per group:
//!
//!   file  kind(P|C)  info_flat  prem_flats  concl_flats  cells
//!
//! where `cells` is `|`-joined per-cell entries `flat:STATUS` with STATUS one of
//!   F<lo>        cell does not wrap; it matches at every L >= lo (lo = min)
//!   lo-hi[,..]   cell wraps; the L runs at which the engine output == bytes
//!   NONE         cell wraps; no L in [8, fitL] reproduces it (cell-doc gap)
//!
//! Groups dumped: multi-cell groups with any wrapping cell OR flat total > 80
//! (boundary/false-positive evidence), and single-cell wrapping groups.
//! Usage: band_dump <corpus_dir>... > bands.tsv

use graph_clean::doclayout::wrap_cell_dot_lr;
use std::collections::HashMap;

fn split_top(s: &str, sep: char) -> Vec<String> {
    let ch: Vec<char> = s.chars().collect();
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut i = 0;
    while i < ch.len() {
        let c = ch[i];
        if c == '\\' && i + 1 < ch.len() {
            cur.push(c);
            cur.push(ch[i + 1]);
            i += 2;
            continue;
        }
        match c {
            '{' => {
                depth += 1;
                cur.push(c);
            }
            '}' => {
                depth -= 1;
                cur.push(c);
            }
            _ if c == sep && depth == 0 => parts.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
        i += 1;
    }
    parts.push(cur);
    parts
}
fn unescape(t: &str) -> String {
    t.replace("\\<", "<").replace("\\>", ">").replace("\\{", "{").replace("\\}", "}").replace("\\|", "|")
}
fn parse_record(label: &str) -> Option<Vec<Vec<String>>> {
    if !(label.starts_with('{') && label.ends_with('}')) {
        return None;
    }
    let inner = &label[1..label.len() - 1];
    let mut groups = Vec::new();
    for g in split_top(inner, '|') {
        let g = g.trim();
        if !(g.starts_with('{') && g.ends_with('}')) {
            return None;
        }
        groups.push(split_top(&g[1..g.len() - 1], '|'));
    }
    Some(groups)
}
fn strip_port(cell: &str) -> Option<&str> {
    let rest = cell.strip_prefix("<n")?;
    let sp = rest.find("> ")?;
    if !rest[..sp].chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(&rest[sp + 2..])
}
fn is_info_body(b: &str) -> bool {
    b.starts_with('#') && b.contains(" : ")
}
fn dewrap(b: &str) -> String {
    // `,\l` = a separator break (the trailing space is dropped at line end);
    // `\l&nbsp;` = an indented continuation or an indented closer peel (no
    // character lost); a remaining col-0 `\l)` = the fact-paren peel (its
    // padded ` )` space was the break). Order matters.
    let t = b.replace(",\\l", ", ");
    let t = t.replace("\\l&nbsp;", "&nbsp;");
    let t = t.replace("\\l)", " )");
    let t = t.replace("\\l", "").replace("&nbsp;", "");
    unescape(&t)
}
fn flat_width(b: &str) -> usize {
    dewrap(b).chars().count()
}

/// Smallest L (line length at ribbons 1.5) whose ribbon >= flat.
fn fit_l(flat: usize) -> isize {
    let mut l = ((flat as f64) * 1.5).floor() as isize - 2;
    if l < 8 {
        l = 8;
    }
    loop {
        // banker's round of L/1.5, as the engine computes the ribbon
        let v = l as f64 / 1.5;
        let fl = v.floor();
        let r = if (v - fl - 0.5).abs() < 1e-9 {
            let f = fl as i64;
            if f % 2 == 0 {
                f
            } else {
                f + 1
            }
        } else {
            v.round() as i64
        };
        if r >= flat as i64 {
            return l;
        }
        l += 1;
    }
}

/// STATUS string for one cell (see module docs).
fn cell_status(flat_text: &str, body: &str, flat: usize) -> String {
    let wraps = body.contains("\\l");
    if !wraps {
        // valid L set is [lo, +inf) (monotone: below the fit boundary the doc
        // wraps, above it renders flat; an unbreakable atom matches everywhere)
        let hi = fit_l(flat) + 2;
        let matches = |l: isize| wrap_cell_dot_lr(flat_text, l, 1.5) == body;
        if matches(8) {
            return "F8".to_string();
        }
        let (mut lo, mut hi) = (8isize, hi);
        // invariant: !matches(lo), matches(hi)
        if !matches(hi) {
            return "FNONE".to_string();
        }
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            if matches(mid) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        return format!("F{}", hi);
    }
    // wrapping: sweep 8 ..= fitL (at/above fitL the render is one line != body)
    let top = fit_l(flat);
    let mut runs: Vec<(isize, isize)> = Vec::new();
    let mut cur: Option<(isize, isize)> = None;
    for l in 8..=top {
        if wrap_cell_dot_lr(flat_text, l, 1.5) == body {
            cur = Some(match cur {
                Some((a, _)) => (a, l),
                None => (l, l),
            });
        } else if let Some(r) = cur.take() {
            runs.push(r);
        }
    }
    if let Some(r) = cur {
        runs.push(r);
    }
    if runs.is_empty() {
        return "NONE".to_string();
    }
    runs.iter().map(|(a, b)| format!("{}-{}", a, b)).collect::<Vec<_>>().join(",")
}

/// Shape features of a cell's flat text, for occupancy-model fitting:
/// `(dtop, drec, nq, sqa, nargs, nfunc, nabbr, ctup, bmax)` where `dtop` = Σ
/// over TOP-LEVEL tuple args of (2·elems − 4), `drec` = the same over ALL
/// tuple nodes recursively, `nq` = number of single-quoted constants anywhere,
/// `sqa` = 1 if the cell is a fact with exactly one argument that is a quoted
/// constant, `nargs` = top-level argument count (0 if the text is not a padded
/// fact), `ctup` = Σ over top-level tuple AND `(a++b)`-union args of
/// (elems + 1) (the round-10 occupancy law), `bmax` = max over those args of
/// (elems/2 + 2) (the round-10 self-budget bonus law).
fn shape_features(flat: &str) -> (i64, i64, i64, i64, i64, i64, i64, i64, i64) {
    fn split_args(s: &str) -> Vec<&str> {
        let mut parts = Vec::new();
        let (mut depth, mut inq, mut start) = (0i32, false, 0usize);
        let b: Vec<(usize, char)> = s.char_indices().collect();
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
    fn tuple_nodes(s: &str, top: bool, dtop: &mut i64, drec: &mut i64) {
        let t = s.trim();
        if t.starts_with('<') && t.ends_with('>') {
            let inner = &t[1..t.len() - 1];
            let elems = split_args(inner);
            let d = 2 * elems.len() as i64 - 4;
            *drec += d;
            if top {
                *dtop += d;
            }
            for e in elems {
                tuple_nodes(e, false, dtop, drec);
            }
        } else if let Some(open) = t.find('(') {
            if t.ends_with(')') {
                let inner = &t[open + 1..t.len() - 1];
                let inner = inner.strip_prefix(' ').unwrap_or(inner);
                let inner = inner.strip_suffix(' ').unwrap_or(inner);
                for a in split_args(inner) {
                    tuple_nodes(a, false, dtop, drec);
                }
            }
        }
    }
    /// Union arg `(a++b++…)`: element count if `t` is a parenthesized
    /// top-level `++` chain, else 0.
    fn union_elems(t: &str) -> i64 {
        let t = t.trim();
        if !(t.starts_with('(') && t.ends_with(')')) {
            return 0;
        }
        let inner = &t[1..t.len() - 1];
        let b: Vec<(usize, char)> = inner.char_indices().collect();
        let (mut depth, mut inq, mut n) = (0i32, false, 1i64);
        let mut i = 0;
        while i < b.len() {
            let c = b[i].1;
            if inq {
                if c == '\'' {
                    inq = false;
                }
            } else {
                match c {
                    '\'' => inq = true,
                    '(' | '<' | '[' => depth += 1,
                    ')' | '>' | ']' => depth -= 1,
                    '+' if depth == 0 && i + 1 < b.len() && b[i + 1].1 == '+' => {
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
    let nq = flat.split('\'').count() as i64 / 2;
    let (mut dtop, mut drec) = (0i64, 0i64);
    let mut nargs = 0i64;
    let mut sqa = 0i64;
    let (mut ctup, mut bmax) = (0i64, 0i64);
    // function-application nodes inside the fact (name directly followed by
    // '(' with no ' ' after — the unpadded display form), e.g. senc(, pk(
    let nfunc = {
        let b: Vec<char> = flat.chars().collect();
        let mut n = 0i64;
        for i in 1..b.len() {
            if b[i] == '('
                && (b[i - 1].is_alphanumeric() || b[i - 1] == '_')
                && (i + 1 >= b.len() || b[i + 1] != ' ')
            {
                n += 1;
            }
        }
        n
    };
    // abbreviation-name tokens (2 uppercase letters + digits, e.g. KD19, EX1)
    let nabbr = {
        let b: Vec<char> = flat.chars().collect();
        let mut n = 0i64;
        let mut i = 0;
        while i < b.len() {
            if b[i].is_ascii_uppercase()
                && (i == 0 || !(b[i - 1].is_alphanumeric() || b[i - 1] == '_'))
            {
                let mut j = i + 1;
                while j < b.len() && b[j].is_ascii_uppercase() {
                    j += 1;
                }
                let letters = j - i;
                let ds = j;
                while j < b.len() && b[j].is_ascii_digit() {
                    j += 1;
                }
                if letters <= 2 && j > ds && (j >= b.len() || !(b[j].is_alphanumeric() || b[j] == '_')) {
                    n += 1;
                }
                i = j;
            } else {
                i += 1;
            }
        }
        n
    };
    if let Some(open) = flat.find("( ") {
        if flat.ends_with(" )") {
            let name = &flat[..open];
            if !name.is_empty() && !name.contains(['(', ')', '<', '>', ' ', ',']) {
                let inner = &flat[open + 2..flat.len() - 2];
                let args = split_args(inner);
                nargs = args.len() as i64;
                for a in &args {
                    tuple_nodes(a, true, &mut dtop, &mut drec);
                    let t = a.trim();
                    let elems = if t.starts_with('<') && t.ends_with('>') {
                        split_args(&t[1..t.len() - 1]).len() as i64
                    } else {
                        union_elems(t)
                    };
                    if elems >= 2 {
                        ctup += elems + 1;
                        bmax = bmax.max(elems / 2 + 2);
                    }
                }
                if args.len() == 1 {
                    let a = args[0].trim();
                    if a.starts_with('\'') && a.ends_with('\'') && a.len() >= 2
                        && !a[1..a.len() - 1].contains('\'')
                    {
                        sqa = 1;
                    }
                }
            }
        }
    }
    (dtop, drec, nq, sqa, nargs, nfunc, nabbr, ctup, bmax)
}

fn process_file(path: &std::path::Path, cache: &mut HashMap<(String, String), String>) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(dot) = std::fs::read_to_string(path) else { return out };
    let base = path.file_name().unwrap().to_string_lossy().to_string();
    for line in dot.lines() {
        if !line.contains("shape=\"record\"") {
            continue;
        }
        let Some(ls) = line.find("label=\"") else { continue };
        let after = &line[ls + 7..];
        let Some(le) = after.find("\",fillcolor") else { continue };
        let Some(groups) = parse_record(&after[..le]) else { continue };
        // locate the info group; groups before = P, after = C
        let mut info_idx = None;
        let mut info_flat = 0usize;
        for (gi, cells) in groups.iter().enumerate() {
            if cells.len() == 1 {
                if let Some(b) = strip_port(&cells[0]) {
                    if is_info_body(b) {
                        info_idx = Some(gi);
                        info_flat = flat_width(b);
                    }
                }
            }
        }
        let Some(info_idx) = info_idx else { continue };
        let group_flats: Vec<Option<Vec<usize>>> = groups
            .iter()
            .map(|cells| {
                cells
                    .iter()
                    .map(|c| strip_port(c).map(flat_width))
                    .collect::<Option<Vec<usize>>>()
            })
            .collect();
        let flats_str = |gi: usize| -> String {
            if gi >= groups.len() {
                return String::new();
            }
            match &group_flats[gi] {
                Some(v) => v.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","),
                None => String::new(),
            }
        };
        // premise group (if any) is the group before info; conclusion after
        let prem_gi = if info_idx > 0 { Some(info_idx - 1) } else { None };
        let concl_gi = if info_idx + 1 < groups.len() { Some(info_idx + 1) } else { None };
        for (kind, gi) in [("P", prem_gi), ("C", concl_gi)] {
            let Some(gi) = gi else { continue };
            let cells = &groups[gi];
            let bodies: Vec<&str> = match cells.iter().map(|c| strip_port(c)).collect::<Option<Vec<_>>>() {
                Some(b) => b,
                None => continue,
            };
            if bodies.iter().any(|b| is_info_body(b)) {
                continue;
            }
            let flats: Vec<usize> = bodies.iter().map(|b| flat_width(b)).collect();
            let total: usize = flats.iter().sum();
            let any_wrap = bodies.iter().any(|b| b.contains("\\l"));
            let dump = if bodies.len() >= 2 {
                any_wrap || total > 80
            } else {
                any_wrap
            };
            if !dump {
                continue;
            }
            let mut cellfields = Vec::with_capacity(bodies.len());
            for (k, body) in bodies.iter().enumerate() {
                let flat_text = dewrap(body);
                let key = (flat_text.clone(), body.to_string());
                let status = if let Some(s) = cache.get(&key) {
                    s.clone()
                } else {
                    let s = cell_status(&flat_text, body, flats[k]);
                    cache.insert(key, s.clone());
                    s
                };
                let (dtop, drec, nq, sqa, nargs, nfunc, nabbr, ctup, bmax) = shape_features(&flat_text);
                // fields 12-16 (round 11): the crate's own separate tuple/union
                // occupancy surcharges, its (capped) bonus, the RECURSIVE
                // surcharge and the fact's top-level arg count, so offline
                // fitting uses the exact shipped shape features
                let cs = graph_clean::generate::cell_shape(&flat_text);
                cellfields.push(format!(
                    "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
                    flats[k], status, dtop, drec, nq, sqa, nargs, nfunc, nabbr, ctup, bmax,
                    cs.tup_sur, cs.uni_sur, cs.bmax, cs.rec_sur, cs.nargs,
                    cs.last_tup as i64,
                    graph_clean::generate::rec_surcharge_capped(&flat_text, 5),
                    graph_clean::generate::rec_surcharge_capped(&flat_text, 7),
                    graph_clean::generate::rec_surcharge_capped(&flat_text, 9)
                ));
            }
            out.push(format!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                base,
                kind,
                info_flat,
                prem_gi.map(flats_str).unwrap_or_default(),
                concl_gi.map(flats_str).unwrap_or_default(),
                cellfields.join("|")
            ));
        }
    }
    out
}

fn main() {
    let dirs: Vec<String> = std::env::args().skip(1).collect();
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for d in &dirs {
        for e in std::fs::read_dir(d).unwrap() {
            let p = e.unwrap().path();
            if p.extension().and_then(|e| e.to_str()) == Some("dot") {
                files.push(p);
            }
        }
    }
    files.sort();
    let nthreads = 8usize;
    let chunks: Vec<Vec<std::path::PathBuf>> = (0..nthreads)
        .map(|t| files.iter().skip(t).step_by(nthreads).cloned().collect())
        .collect();
    let handles: Vec<_> = chunks
        .into_iter()
        .map(|chunk| {
            std::thread::spawn(move || {
                let mut cache = HashMap::new();
                let mut out = Vec::new();
                for p in chunk {
                    out.extend(process_file(&p, &mut cache));
                }
                out
            })
        })
        .collect();
    let mut all = Vec::new();
    for h in handles {
        all.extend(h.join().unwrap());
    }
    all.sort();
    let mut so = String::new();
    for l in all {
        so.push_str(&l);
        so.push('\n');
    }
    print!("{}", so);
}

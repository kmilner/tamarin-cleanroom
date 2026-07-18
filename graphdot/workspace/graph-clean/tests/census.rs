//! Corpus census (off by default): measure how faithfully the engine-based cell
//! layout reproduces the captured record cells.
//!
//! Set `GRAPHCLEAN_CORPUS` to a directory of `*.dot` payloads. For every record
//! in the corpus we parse its groups/cells, DEWRAP each wrapped cell back to its
//! flat text, re-lay the group out through the faithful engine
//! (`generate::group_widths` + `doclayout::wrap_cell_dot`), and compare byte-for-
//! byte. Reports the match rate over wrapping premise/conclusion cells (the metric
//! the closed-form model reached 44 % on), over info cells, and over all records.

use graph_clean::doclayout::wrap_cell_dot;
use graph_clean::generate::group_widths;

/// Split a record-label body `{...}|{...}|{...}` (already stripped of the outer
/// braces) at top-level `|`, honoring unescaped `{ }` nesting; escaped `\{ \} \|`
/// inside cell text are literal and ignored.
fn split_top_pipes(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let bytes: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == '\\' && i + 1 < bytes.len() {
            cur.push(c);
            cur.push(bytes[i + 1]);
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
            '|' if depth == 0 => {
                out.push(std::mem::take(&mut cur));
            }
            _ => cur.push(c),
        }
        i += 1;
    }
    out.push(cur);
    out
}

fn strip_braces(s: &str) -> &str {
    let s = s.trim();
    let s = s.strip_prefix('{').unwrap_or(s);
    s.strip_suffix('}').unwrap_or(s)
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == '\\' && i + 1 < cs.len() && matches!(cs[i + 1], '<' | '>' | '{' | '}' | '|') {
            out.push(cs[i + 1]);
            i += 2;
        } else {
            out.push(cs[i]);
            i += 1;
        }
    }
    out
}

/// Strip a leading port marker `<nK> ` from a cell.
fn strip_port(cell: &str) -> &str {
    let c = cell.trim_start();
    if let Some(rest) = c.strip_prefix('<') {
        if let Some(gt) = rest.find('>') {
            let after = &rest[gt + 1..];
            return after.strip_prefix(' ').unwrap_or(after);
        }
    }
    c
}

/// Dewrap a cell's wrapped bytes (with `\l` / `&nbsp;`, escaped) back to its flat,
/// un-escaped text, inverting the observed break joins (BEHAVIOR.md §3f):
/// a bare `,` line-end regains its dropped space, a `, ` line-end joins directly,
/// a peeled `>` joins directly, a peeled `)` regains its ` `.
fn dewrap(cell: &str) -> String {
    if !cell.contains("\\l") {
        return unescape(cell);
    }
    let raw_lines: Vec<&str> = cell.split("\\l").collect();
    // Drop the trailing empty piece after the final "\l".
    let mut lines: Vec<String> = Vec::new();
    for (i, l) in raw_lines.iter().enumerate() {
        if i + 1 == raw_lines.len() && l.is_empty() {
            break;
        }
        // Strip leading &nbsp; runs.
        let mut rest = *l;
        while let Some(r) = rest.strip_prefix("&nbsp;") {
            rest = r;
        }
        lines.push(unescape(rest));
    }
    if lines.is_empty() {
        return String::new();
    }
    let mut flat = lines[0].clone();
    for li in &lines[1..] {
        if flat.ends_with(", ") {
            flat.push_str(li);
        } else if flat.ends_with(',') {
            flat.push(' ');
            flat.push_str(li);
        } else if li == ">" {
            flat.push_str(li);
        } else if li == ")" {
            flat.push_str(" )");
        } else {
            flat.push_str(li);
        }
    }
    flat
}

struct RecordCells {
    premises: Vec<String>,
    info: String,
    conclusions: Vec<String>,
}

/// Parse a record label body into flat premise/info/conclusion cells (dewrapped).
fn parse_record(label_body: &str) -> Option<RecordCells> {
    let inner = strip_braces(label_body); // remove outer { }
    let groups = split_top_pipes(inner);
    // Each group is `{cell|cell|...}`; the info group is the one whose cell starts
    // with `#`.
    let mut parsed: Vec<Vec<String>> = Vec::new();
    for g in &groups {
        let gi = strip_braces(g);
        let cells = split_top_pipes(gi);
        let flats: Vec<String> = cells.iter().map(|c| dewrap(strip_port(c))).collect();
        parsed.push(flats);
    }
    // Locate the info group.
    let mut info_idx = None;
    for (i, g) in parsed.iter().enumerate() {
        if g.first().map(|c| c.starts_with('#')).unwrap_or(false) {
            info_idx = Some(i);
            break;
        }
    }
    let info_idx = info_idx?;
    let info = parsed[info_idx].first()?.clone();
    let premises: Vec<String> = parsed[..info_idx].iter().flatten().cloned().collect();
    let conclusions: Vec<String> = parsed[info_idx + 1..].iter().flatten().cloned().collect();
    Some(RecordCells { premises, info, conclusions })
}

/// Extract each record's raw `label="…"` body and the raw group/cell wrapped
/// strings, without dewrapping, so we can compare re-layout to the original.
fn each_record_label(dot: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in dot.lines() {
        if !line.contains("shape=\"record\"") {
            continue;
        }
        if let Some(start) = line.find("label=\"") {
            let after = &line[start + 7..];
            // label ends at `",fillcolor=`
            if let Some(end) = after.find("\",fillcolor=") {
                out.push(after[..end].to_string());
            }
        }
    }
    out
}

/// The raw wrapped cells of one record group, keyed premises/info/conclusions,
/// preserving the original bytes (with `\l`, `&nbsp;`, escapes, minus the port).
struct RawRecord {
    premises: Vec<String>,
    info: String,
    conclusions: Vec<String>,
}

fn parse_raw(label_body: &str) -> Option<RawRecord> {
    let inner = strip_braces(label_body);
    let groups = split_top_pipes(inner);
    let mut parsed: Vec<Vec<String>> = Vec::new();
    for g in &groups {
        let gi = strip_braces(g);
        let cells = split_top_pipes(gi);
        let raws: Vec<String> = cells.iter().map(|c| strip_port(c).to_string()).collect();
        parsed.push(raws);
    }
    let mut info_idx = None;
    for (i, g) in parsed.iter().enumerate() {
        // info cell (raw) starts with `#` (possibly after nothing).
        if g.first().map(|c| c.trim_start().starts_with('#')).unwrap_or(false) {
            info_idx = Some(i);
            break;
        }
    }
    let info_idx = info_idx?;
    let info = parsed[info_idx].first()?.clone();
    let premises: Vec<String> = parsed[..info_idx].iter().flatten().cloned().collect();
    let conclusions: Vec<String> = parsed[info_idx + 1..].iter().flatten().cloned().collect();
    Some(RawRecord { premises, info, conclusions })
}

#[test]
#[ignore]
fn corpus_census() {
    let Ok(dir) = std::env::var("GRAPHCLEAN_CORPUS") else {
        eprintln!("GRAPHCLEAN_CORPUS not set; skipping");
        return;
    };
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .expect("read corpus dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "dot").unwrap_or(false))
        .collect();
    files.sort();

    let mut wrap_cells = 0u64; // wrapping prem/concl cells
    let mut wrap_ok = 0u64;
    let mut all_pc = 0u64; // all prem/concl cells
    let mut all_pc_ok = 0u64;
    let mut info_cells = 0u64;
    let mut info_ok = 0u64;
    let mut records = 0u64;
    let mut records_ok = 0u64;

    for path in &files {
        let dot = std::fs::read_to_string(path).unwrap();
        for label in each_record_label(&dot) {
            let (Some(flat), Some(raw)) = (parse_record(&label), parse_raw(&label)) else {
                continue;
            };
            if flat.premises.len() != raw.premises.len()
                || flat.conclusions.len() != raw.conclusions.len()
            {
                continue;
            }
            records += 1;
            let mut rec_ok = true;

            // info
            let info_got = wrap_cell_dot(&flat.info, 87);
            info_cells += 1;
            if info_got == raw.info {
                info_ok += 1;
            } else {
                rec_ok = false;
            }

            for (flats, raws) in [
                (&flat.premises, &raw.premises),
                (&flat.conclusions, &raw.conclusions),
            ] {
                let ws = group_widths(flats);
                for ((f, r), w) in flats.iter().zip(raws).zip(ws) {
                    let got = wrap_cell_dot(f, w as isize);
                    all_pc += 1;
                    let is_wrap = r.contains("\\l");
                    if is_wrap {
                        wrap_cells += 1;
                    }
                    if &got == r {
                        all_pc_ok += 1;
                        if is_wrap {
                            wrap_ok += 1;
                        }
                    } else {
                        rec_ok = false;
                    }
                }
            }
            if rec_ok {
                records_ok += 1;
            }
        }
    }

    let pct = |a: u64, b: u64| if b == 0 { 0.0 } else { 100.0 * a as f64 / b as f64 };
    eprintln!("=== CENSUS over {} files ===", files.len());
    eprintln!(
        "wrapping prem/concl cells: {}/{} = {:.3}%",
        wrap_ok, wrap_cells, pct(wrap_ok, wrap_cells)
    );
    eprintln!(
        "ALL prem/concl cells:      {}/{} = {:.3}%",
        all_pc_ok, all_pc, pct(all_pc_ok, all_pc)
    );
    eprintln!(
        "info cells:                {}/{} = {:.3}%",
        info_ok, info_cells, pct(info_ok, info_cells)
    );
    eprintln!(
        "records (all cells exact): {}/{} = {:.3}%",
        records_ok, records, pct(records_ok, records)
    );
}

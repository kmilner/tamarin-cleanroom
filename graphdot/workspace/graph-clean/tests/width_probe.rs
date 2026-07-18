//! Measure the CEILING for a width-based model: fraction of multi-cell wrapping
//! cells reproducible at SOME width (non-empty band). Also test candidate
//! allocators against the per-cell band. Ignored; GRAPHCLEAN_CORPUS set.

use graph_clean::doclayout::wrap_cell_dot;

fn split_top(s: &str, sep: char) -> Vec<String> {
    let ch: Vec<char> = s.chars().collect();
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut i = 0;
    while i < ch.len() {
        let c = ch[i];
        if c == '\\' && i + 1 < ch.len() { cur.push(c); cur.push(ch[i + 1]); i += 2; continue; }
        match c {
            '{' => { depth += 1; cur.push(c); }
            '}' => { depth -= 1; cur.push(c); }
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
    if !(label.starts_with('{') && label.ends_with('}')) { return None; }
    let inner = &label[1..label.len() - 1];
    let mut groups = Vec::new();
    for g in split_top(inner, '|') {
        let g = g.trim();
        if !(g.starts_with('{') && g.ends_with('}')) { return None; }
        groups.push(split_top(&g[1..g.len() - 1], '|'));
    }
    Some(groups)
}
fn strip_port(cell: &str) -> Option<&str> {
    let rest = cell.strip_prefix("<n")?;
    let sp = rest.find("> ")?;
    if !rest[..sp].chars().all(|c| c.is_ascii_digit()) { return None; }
    Some(&rest[sp + 2..])
}
fn is_info_body(b: &str) -> bool { b.starts_with('#') && b.contains(" : ") }
fn dewrap(b: &str) -> String {
    unescape(&b.replace("&nbsp;", "").replace(",\\l", ", ").replace("\\l)", " )").replace("\\l", ""))
}
fn flat_width(b: &str) -> usize {
    let lost = b.matches(",\\l").count() + b.matches("\\l)").count();
    unescape(&b.replace("\\l", "").replace("&nbsp;", "")).chars().count() + lost
}
fn band(flat: &str, body: &str) -> (usize, usize) {
    let mut lo = 0;
    let mut hi = 0;
    for w in 15..=95isize {
        if wrap_cell_dot(flat, w) == body {
            if lo == 0 { lo = w as usize; }
            hi = w as usize;
        }
    }
    (lo, hi)
}

// allocators
fn a_prop(flats: &[usize], i: usize) -> usize {
    let t: usize = flats.iter().sum();
    ((87 * flats[i]) as f64 / t as f64).round().max(20.0) as usize
}
fn a_clamped(flats: &[usize], i: usize, c: usize) -> usize {
    // 87 - sum over others of min(flat_j, c), floor 20
    let s: usize = flats.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, &f)| f.min(c)).sum();
    87usize.saturating_sub(s).max(20)
}

#[test]
#[ignore]
fn width_probe() {
    let Ok(corpus) = std::env::var("GRAPHCLEAN_CORPUS") else { return };
    let sample: usize = std::env::var("SAMPLE").ok().and_then(|s| s.parse().ok()).unwrap_or(4000);
    let mut files: Vec<_> = std::fs::read_dir(&corpus).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    // sample across the whole corpus, not just the first files
    let step = (files.len() / 400).max(1);

    let mut multi_wrap = 0usize;
    let mut has_band = 0usize;
    // candidate hits
    let mut hit_prop = 0usize;
    let mut hit_c8 = 0usize;
    let mut hit_c12 = 0usize;
    let mut hit_c16 = 0usize;
    let mut hit_c20 = 0usize;
    let mut hit_best = 0usize; // any width in band (ceiling)
    let mut done = 0usize;

    'outer: for (fi, path) in files.iter().enumerate() {
        if fi % step != 0 { continue; }
        if path.extension().and_then(|e| e.to_str()) != Some("dot") { continue; }
        let dot = std::fs::read_to_string(path).unwrap();
        for line in dot.lines() {
            if !line.contains("shape=\"record\"") { continue; }
            let Some(ls) = line.find("label=\"") else { continue };
            let after = &line[ls + 7..];
            let Some(le) = after.find("\",fillcolor") else { continue };
            let Some(groups) = parse_record(&after[..le]) else { continue };
            for cells in &groups {
                let bodies: Vec<&str> = match cells.iter().map(|c| strip_port(c)).collect::<Option<Vec<_>>>() {
                    Some(b) => b, None => continue,
                };
                if bodies.len() < 2 || bodies.iter().any(|b| is_info_body(b)) { continue; }
                let flats: Vec<usize> = bodies.iter().map(|b| flat_width(b)).collect();
                for (k, body) in bodies.iter().enumerate() {
                    if !body.contains("\\l") { continue; }
                    multi_wrap += 1;
                    let flat = dewrap(body);
                    let (lo, hi) = band(&flat, body);
                    let inband = |w: usize| lo > 0 && lo <= w && w <= hi;
                    if lo > 0 { has_band += 1; hit_best += 1; }
                    if inband(a_prop(&flats, k)) { hit_prop += 1; }
                    if inband(a_clamped(&flats, k, 8)) { hit_c8 += 1; }
                    if inband(a_clamped(&flats, k, 12)) { hit_c12 += 1; }
                    if inband(a_clamped(&flats, k, 16)) { hit_c16 += 1; }
                    if inband(a_clamped(&flats, k, 20)) { hit_c20 += 1; }
                    done += 1;
                    if done >= sample { break 'outer; }
                }
            }
        }
    }
    let p = |x: usize| 100.0 * x as f64 / multi_wrap.max(1) as f64;
    eprintln!("multi-cell wrapping cells sampled: {multi_wrap}");
    eprintln!("  reproducible at SOME width (ceiling): {has_band} = {:.1}%", p(has_band));
    eprintln!("  proportional(87):   {hit_prop} = {:.1}%", p(hit_prop));
    eprintln!("  clamped C=8:        {hit_c8} = {:.1}%", p(hit_c8));
    eprintln!("  clamped C=12:       {hit_c12} = {:.1}%", p(hit_c12));
    eprintln!("  clamped C=16:       {hit_c16} = {:.1}%", p(hit_c16));
    eprintln!("  clamped C=20:       {hit_c20} = {:.1}%", p(hit_c20));
}

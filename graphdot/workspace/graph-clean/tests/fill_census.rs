//! Corpus census of the record-cell wrap model (BEHAVIOR.md §3f).
//!
//! For every record in the corpus, parse its label into groups → cells, dewrap
//! each cell back to its flat (post-abbreviation) text, re-run several candidate
//! per-cell width allocators through the exact layout engine, and compare the
//! produced record-label bytes to the actual cell bytes. Reports byte-exactness
//! over wrapping prem/concl cells (the round-8 metric) and over all cells, per
//! allocator, in ONE corpus pass.
//!
//! Ignored by default; run with `GRAPHCLEAN_CORPUS=<dir> cargo test --release
//! --test fill_census -- --ignored --nocapture census`.

use graph_clean::doclayout::wrap_cell_dot;

fn split_top(s: &str, sep: char) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            cur.push(c);
            cur.push(chars[i + 1]);
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
        let gi = &g[1..g.len() - 1];
        groups.push(split_top(gi, '|'));
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

fn is_info_body(body: &str) -> bool {
    body.starts_with('#') && body.contains(" : ")
}

fn dewrap(body: &str) -> String {
    let t = body.replace("&nbsp;", "");
    let t = t.replace(",\\l", ", ");
    let t = t.replace("\\l)", " )");
    let t = t.replace("\\l", "");
    unescape(&t)
}

fn flat_width(body: &str) -> usize {
    let lost = body.matches(",\\l").count() + body.matches("\\l)").count();
    let bare = unescape(&body.replace("\\l", "").replace("&nbsp;", ""));
    bare.chars().count() + lost
}

const FLOOR: usize = 20;
const W: usize = 87;

// ---- candidate allocators: given the group's cell flats, per-cell width ----

fn alloc_smallest_first(flats: &[usize]) -> Vec<usize> {
    let n = flats.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| flats[i]);
    let mut occ = flats.to_vec();
    let mut out = vec![0usize; n];
    for &i in &order {
        let others: usize = (0..n).filter(|&j| j != i).map(|j| occ[j]).sum();
        let b = W.saturating_sub(others).max(FLOOR);
        out[i] = b;
        occ[i] = flats[i].min(b);
    }
    out
}

fn alloc_flatsum(flats: &[usize]) -> Vec<usize> {
    let t: usize = flats.iter().sum();
    (0..flats.len())
        .map(|i| W.saturating_sub(t - flats[i]).max(FLOOR))
        .collect()
}

fn alloc_proportional(flats: &[usize]) -> Vec<usize> {
    let t: usize = flats.iter().sum();
    if t == 0 {
        return vec![W; flats.len()];
    }
    flats
        .iter()
        .map(|&f| ((W * f + t / 2) / t).max(FLOOR))
        .collect()
}

fn alloc_prop_l(flats: &[usize], l: usize) -> Vec<usize> {
    let t: usize = flats.iter().sum();
    if t == 0 {
        return vec![W; flats.len()];
    }
    flats.iter().map(|&f| ((l * f + t / 2) / t).max(FLOOR)).collect()
}

fn alloc_prop_ceil(flats: &[usize]) -> Vec<usize> {
    let t: usize = flats.iter().sum();
    if t == 0 {
        return vec![W; flats.len()];
    }
    flats.iter().map(|&f| ((W * f + t - 1) / t).max(FLOOR)).collect()
}

/// Reserve cells whose flat ≤ FLOOR (they never wrap) at their flat width, then
/// distribute the remaining budget proportionally among the larger cells.
fn alloc_reserve_small(flats: &[usize]) -> Vec<usize> {
    let reserved: usize = flats.iter().filter(|&&f| f <= FLOOR).sum();
    let big_sum: usize = flats.iter().filter(|&&f| f > FLOOR).sum();
    if big_sum == 0 {
        return flats.iter().map(|&f| f.max(FLOOR)).collect();
    }
    let avail = W.saturating_sub(reserved).max(FLOOR);
    flats
        .iter()
        .map(|&f| {
            if f <= FLOOR {
                FLOOR
            } else {
                ((avail * f + big_sum / 2) / big_sum).max(FLOOR)
            }
        })
        .collect()
}

struct Tally {
    name: String,
    wrap_total: usize,
    wrap_match: usize,
    all_total: usize,
    all_match: usize,
    // diagnostics (single vs multi cell groups; failure modes)
    single_wrap_total: usize,
    single_wrap_match: usize,
    multi_wrap_total: usize,
    multi_wrap_match: usize,
    // mismatch modes on wrapping cells
    fn_predict_oneline: usize, // actual wraps, predicted single line (false neg)
    fill_error: usize,         // both wrap, differ
    fp_examples: Vec<(String, usize, String, String)>, // flat, width, want, got
}

fn main_census() {
    let Ok(corpus) = std::env::var("GRAPHCLEAN_CORPUS") else {
        eprintln!("set GRAPHCLEAN_CORPUS");
        return;
    };
    type Alloc = (&'static str, Box<dyn Fn(&[usize]) -> Vec<usize>>);
    let allocs: Vec<Alloc> = vec![
        ("proportional(87)", Box::new(alloc_proportional)),
        ("prop_ceil", Box::new(alloc_prop_ceil)),
        ("reserve_small", Box::new(alloc_reserve_small)),
        ("prop(89)", Box::new(|f: &[usize]| alloc_prop_l(f, 89))),
    ];
    let _ = (alloc_smallest_first, alloc_flatsum, alloc_prop_l);
    let mut tallies: Vec<Tally> = allocs
        .iter()
        .map(|(n, _)| Tally {
            name: n.to_string(),
            wrap_total: 0,
            wrap_match: 0,
            all_total: 0,
            all_match: 0,
            single_wrap_total: 0,
            single_wrap_match: 0,
            multi_wrap_total: 0,
            multi_wrap_match: 0,
            fn_predict_oneline: 0,
            fill_error: 0,
            fp_examples: Vec::new(),
        })
        .collect();

    let step: usize = std::env::var("STEP").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
    for (fi, entry) in std::fs::read_dir(&corpus).unwrap().enumerate() {
        if step > 1 && fi % step != 0 {
            continue;
        }
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("dot") {
            continue;
        }
        let dot = std::fs::read_to_string(&path).unwrap();
        for line in dot.lines() {
            if !line.contains("shape=\"record\"") {
                continue;
            }
            let Some(lstart) = line.find("label=\"") else { continue };
            let after = &line[lstart + 7..];
            let Some(lend) = after.find("\",fillcolor") else { continue };
            let label = &after[..lend];
            let Some(groups) = parse_record(label) else { continue };
            for cells in &groups {
                let bodies: Vec<&str> = match cells.iter().map(|c| strip_port(c)).collect::<Option<Vec<_>>>() {
                    Some(b) => b,
                    None => continue,
                };
                if bodies.iter().any(|b| is_info_body(b)) {
                    continue;
                }
                let flats: Vec<usize> = bodies.iter().map(|b| flat_width(b)).collect();
                let flat_texts: Vec<String> = bodies.iter().map(|b| dewrap(b)).collect();
                let single = bodies.len() == 1;
                for (ti, (_, alloc)) in allocs.iter().enumerate() {
                    let widths = alloc(&flats);
                    for (k, body) in bodies.iter().enumerate() {
                        let predicted = wrap_cell_dot(&flat_texts[k], widths[k] as isize);
                        let is_wrap = body.contains("\\l");
                        let m = predicted == *body;
                        tallies[ti].all_total += 1;
                        if m {
                            tallies[ti].all_match += 1;
                        }
                        if is_wrap {
                            tallies[ti].wrap_total += 1;
                            if m {
                                tallies[ti].wrap_match += 1;
                            }
                            if single {
                                tallies[ti].single_wrap_total += 1;
                                if m {
                                    tallies[ti].single_wrap_match += 1;
                                }
                            } else {
                                tallies[ti].multi_wrap_total += 1;
                                if m {
                                    tallies[ti].multi_wrap_match += 1;
                                }
                            }
                            if !m {
                                if !predicted.contains("\\l") {
                                    tallies[ti].fn_predict_oneline += 1;
                                    if tallies[ti].fp_examples.len() < 25 && single {
                                        tallies[ti].fp_examples.push((
                                            flat_texts[k].clone(),
                                            widths[k],
                                            body.to_string(),
                                            predicted.clone(),
                                        ));
                                    }
                                } else {
                                    tallies[ti].fill_error += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    for t in &tallies {
        eprintln!(
            "{:16} wrap {:>6}/{:<6}={:6.2}% | single {:>6}/{:<6}={:6.2}% | multi {:>6}/{:<6}={:6.2}% | falseNeg {:>5} fillErr {:>5}",
            t.name,
            t.wrap_match,
            t.wrap_total,
            100.0 * t.wrap_match as f64 / t.wrap_total.max(1) as f64,
            t.single_wrap_match,
            t.single_wrap_total,
            100.0 * t.single_wrap_match as f64 / t.single_wrap_total.max(1) as f64,
            t.multi_wrap_match,
            t.multi_wrap_total,
            100.0 * t.multi_wrap_match as f64 / t.multi_wrap_total.max(1) as f64,
            t.fn_predict_oneline,
            t.fill_error,
        );
    }
    if std::env::var("SHOW_FP").is_ok() {
        // single-cell false-negatives for the proportional allocator (index 2)
        for (flat, w, want, got) in tallies[2].fp_examples.iter().take(20) {
            eprintln!("--- single-cell falseNeg  w={w}  flat({}): {flat}", flat.chars().count());
            eprintln!("    want: {want}");
            eprintln!("    got : {got}");
        }
    }
}

#[test]
#[ignore]
fn census() {
    main_census();
}

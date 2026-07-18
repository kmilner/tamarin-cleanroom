//! Round-9 lab: test whether a record GROUP (prem/concl row) is laid out by the
//! reference as ONE HughesPJ document over the cell docs. Renders candidate
//! group compositions (fcat/fsep/cat/sep at candidate line/ribbon settings)
//! with zero-width markers between cells, splits the render at the markers,
//! and compares each cell's line contents against the observed record bytes.
//!
//! Usage: groupdoc_lab  (cases are built in; prints a match matrix)

use graph_clean::doclayout::cell_doc;
use graph_clean::pretty::{beside_op, cat, fcat, fsep, render_page, sep, sized_text, Doc};

struct Case {
    name: &'static str,
    cells: Vec<String>,
    /// expected per-cell physical-line CONTENTS (indentation stripped)
    expect: Vec<Vec<String>>,
}

fn pv(i: usize) -> String {
    let a = b"abcdefghijklmnopqrstuvwxyz";
    format!("${}{}", a[i / 26] as char, a[i % 26] as char)
}
fn pvs(n: usize, off: usize) -> Vec<String> {
    (0..n).map(|i| pv(i + off)).collect()
}
fn argfact(name: &str, n: usize, off: usize) -> String {
    format!("{}( {} )", name, pvs(n, off).join(", "))
}
fn bigtuple(name: &str, n: usize, off: usize) -> String {
    format!("{}( <{}> )", name, pvs(n, off).join(", "))
}
// r8-style 2-char unquoted vars
fn v8(i: usize) -> String {
    let a = b"abcdefghijklmnopqrstuvwxyz";
    format!("{}{}", 'a', a[i % 26] as char)
}
fn big8(n: usize) -> String {
    format!("Big( <{}> )", (0..n).map(v8).collect::<Vec<_>>().join(", "))
}
fn sibc(p: usize, idx: &str) -> String {
    format!("Sib( '{}{}' )", "a".repeat(p), idx)
}

fn lines(s: &[&str]) -> Vec<String> {
    s.iter().map(|x| x.to_string()).collect()
}

fn cases() -> Vec<Case> {
    let mut cs = Vec::new();
    // G_40_40: both flat
    cs.push(Case {
        name: "G_40_40",
        cells: vec![argfact("Faa", 7, 0), argfact("Fbb", 7, 7)],
        expect: vec![
            lines(&["Faa( $aa, $ab, $ac, $ad, $ae, $af, $ag )"]),
            lines(&["Fbb( $ah, $ai, $aj, $ak, $al, $am, $an )"]),
        ],
    });
    // G_45_45: both minimal wrap
    cs.push(Case {
        name: "G_45_45",
        cells: vec![argfact("Faa", 8, 0), argfact("Fbb", 8, 8)],
        expect: vec![
            lines(&["Faa( $aa, $ab, $ac, $ad, $ae, $af, $ag, $ah", ")"]),
            lines(&["Fbb( $ai, $aj, $ak, $al, $am, $an, $ao, $ap", ")"]),
        ],
    });
    // G_50_40
    cs.push(Case {
        name: "G_50_40",
        cells: vec![argfact("Faa", 9, 0), argfact("Fbb", 7, 9)],
        expect: vec![
            lines(&["Faa( $aa, $ab, $ac, $ad, $ae, $af, $ag, $ah, $ai", ")"]),
            lines(&["Fbb( $aj, $ak, $al, $am, $an, $ao, $ap", ")"]),
        ],
    });
    // G_60_30
    cs.push(Case {
        name: "G_60_30",
        cells: vec![argfact("Faa", 11, 0), argfact("Fbb", 5, 11)],
        expect: vec![
            lines(&[
                "Faa( $aa, $ab, $ac, $ad, $ae, $af, $ag, $ah, $ai, $aj, $ak",
                ")",
            ]),
            lines(&["Fbb( $al, $am, $an, $ao, $ap", ")"]),
        ],
    });
    // FW_5_5_5: all three minimal
    cs.push(Case {
        name: "FW_5_5_5",
        cells: vec![argfact("Faa", 5, 0), argfact("Fbb", 5, 5), argfact("Fcc", 5, 10)],
        expect: vec![
            lines(&["Faa( $aa, $ab, $ac, $ad, $ae", ")"]),
            lines(&["Fbb( $af, $ag, $ah, $ai, $aj", ")"]),
            lines(&["Fcc( $ak, $al, $am, $an, $ao", ")"]),
        ],
    });
    // FW_8_8_8: 5 args line0, 3 line1
    cs.push(Case {
        name: "FW_8_8_8",
        cells: vec![argfact("Faa", 8, 0), argfact("Fbb", 8, 8), argfact("Fcc", 8, 16)],
        expect: vec![
            lines(&["Faa( $aa, $ab, $ac, $ad, $ae,", "$af, $ag, $ah", ")"]),
            lines(&["Fbb( $ai, $aj, $ak, $al, $am,", "$an, $ao, $ap", ")"]),
            lines(&["Fcc( $aq, $ar, $as, $at, $au,", "$av, $aw, $ax", ")"]),
        ],
    });
    // FW_8_4_4
    cs.push(Case {
        name: "FW_8_4_4",
        cells: vec![argfact("Faa", 8, 0), argfact("Fbb", 4, 8), argfact("Fcc", 4, 12)],
        expect: vec![
            lines(&["Faa( $aa, $ab, $ac, $ad, $ae, $af, $ag,", "$ah", ")"]),
            lines(&["Fbb( $ai, $aj, $ak, $al", ")"]),
            lines(&["Fcc( $am, $an, $ao, $ap", ")"]),
        ],
    });
    // A series: [Big87 ($-vars 16), Sib33] -> Big 12 elems line0 + 4, sib minimal
    cs.push(Case {
        name: "A_concl",
        cells: vec![bigtuple("Big", 16, 0), sibc(22, "00")],
        expect: vec![
            lines(&[
                "Big( <$aa, $ab, $ac, $ad, $ae, $af, $ag, $ah, $ai, $aj, $ak, $al, ",
                "$am, $an, $ao, $ap>",
                ")",
            ]),
            lines(&["Sib( 'aaaaaaaaaaaaaaaaaaaaaa00'", ")"]),
        ],
    });
    // H_10: sib flat 21 -> Big 13 elems line0
    cs.push(Case {
        name: "H_10",
        cells: vec![bigtuple("Big", 16, 0), sibc(10, "51")],
        expect: vec![
            lines(&[
                "Big( <$aa, $ab, $ac, $ad, $ae, $af, $ag, $ah, $ai, $aj, $ak, $al, $am, ",
                "$an, $ao, $ap>",
                ")",
            ]),
            lines(&["Sib( 'aaaaaaaaaa51'", ")"]),
        ],
    });
    // H_55: sib flat 66 -> Big 9 elems line0
    cs.push(Case {
        name: "H_55",
        cells: vec![bigtuple("Big", 16, 0), sibc(55, "55")],
        expect: vec![
            lines(&[
                "Big( <$aa, $ab, $ac, $ad, $ae, $af, $ag, $ah, $ai, ",
                "$aj, $ak, $al, $am, $an, $ao, $ap>",
                ")",
            ]),
            lines(&[
                "Sib( 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa55'",
                ")",
            ]),
        ],
    });
    // C: lone Bigg (17 elems, flat 93): 16 elems line0 + $aq
    cs.push(Case {
        name: "C_lone",
        cells: vec![bigtuple("Bigg", 17, 0)],
        expect: vec![lines(&[
            "Bigg( <$aa, $ab, $ac, $ad, $ae, $af, $ag, $ah, $ai, $aj, $ak, $al, $am, $an, $ao, $ap, ",
            "$aq>",
            ")",
        ])],
    });
    // r8 l_R_12_20: [Big55 (12 2ch vars), Sib31]: Big FLAT, sib minimal
    cs.push(Case {
        name: "r8_55_31",
        cells: vec![big8(12), sibc(20, "36")],
        expect: vec![
            lines(&["Big( <aa, ab, ac, ad, ae, af, ag, ah, ai, aj, ak, al> )"]),
            lines(&["Sib( 'aaaaaaaaaaaaaaaaaaaa36'", ")"]),
        ],
    });
    // r8 l_R_16_8: [Big71, Sib19]: both flat
    cs.push(Case {
        name: "r8_71_19",
        cells: vec![big8(16), sibc(8, "..")],
        expect: vec![
            lines(&["Big( <aa, ab, ac, ad, ae, af, ag, ah, ai, aj, ak, al, am, an, ao, ap> )"]),
            lines(&["Sib( 'aaaaaaaa..' )"]),
        ],
    });
    // r8 l_R_20_60: [Big87 (20 2ch), Sib71]: Big 11 elems line0
    cs.push(Case {
        name: "r8_87_71",
        cells: vec![big8(20), sibc(60, "69")],
        expect: vec![
            lines(&[
                "Big( <aa, ab, ac, ad, ae, af, ag, ah, ai, aj, ak, ",
                "al, am, an, ao, ap, aq, ar, as, at>",
                ")",
            ]),
            lines(&[
                "Sib( 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa69'",
                ")",
            ]),
        ],
    });
    // r8 l_R_20_60 PREM row: [Fr(~n69) 11, In87]: In 18 elems line0 + 2
    cs.push(Case {
        name: "r8_prem_11_87",
        cells: vec![
            "Fr( ~n69 )".to_string(),
            format!("In( <{}> )", (0..20).map(v8).collect::<Vec<_>>().join(", ")),
        ],
        expect: vec![
            lines(&["Fr( ~n69 )"]),
            lines(&[
                "In( <aa, ab, ac, ad, ae, af, ag, ah, ai, aj, ak, al, am, an, ao, ap, aq, ar, ",
                "as, at>",
                ")",
            ]),
        ],
    });
    cs
}

/// Render the group doc, split at \x01 markers, return per-cell line contents
/// (indentation and trailing spaces are NOT stripped from the interior; lines
/// are split on '\n'; leading spaces of continuation lines are stripped).
fn render_split(op: &str, l: isize, rpl: f64, cells: &[String]) -> Vec<Vec<String>> {
    let docs: Vec<Doc> = cells
        .iter()
        .map(|c| beside_op(sized_text(0, "\u{1}"), cell_doc(c)))
        .collect();
    let g = match op {
        "fcat" => fcat(docs),
        "fsep" => fsep(docs),
        "cat" => cat(docs),
        "sep" => sep(docs),
        _ => unreachable!(),
    };
    let out = render_page(l, rpl, &g);
    // split into per-cell segments on the marker
    let mut per_cell: Vec<String> = Vec::new();
    for (i, seg) in out.split('\u{1}').enumerate() {
        if i == 0 {
            continue; // before first marker (empty or indentation)
        }
        per_cell.push(seg.to_string());
    }
    per_cell
        .iter()
        .map(|seg| {
            seg.split('\n')
                .map(|l| l.trim_start_matches(' ').trim_end_matches(' ').to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .collect()
}

fn norm(v: &[Vec<String>]) -> Vec<Vec<String>> {
    v.iter()
        .map(|c| {
            c.iter()
                .map(|l| l.trim_end_matches(' ').to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .collect()
}

fn main() {
    let cases = cases();
    let ops = ["fcat", "fsep", "cat", "sep"];
    let params: Vec<(isize, f64)> = vec![
        (130, 1.5),
        (100, 1.5),
        (87, 1.5),
        (87, 1.0),
        (100, 1.0),
        (130, 1.0),
        (90, 1.0),
        (90, 1.5),
    ];
    for op in &ops {
        for (l, r) in &params {
            let mut hits = 0;
            let mut names: Vec<&str> = Vec::new();
            for c in &cases {
                let got = render_split(op, *l, *r, &c.cells);
                if norm(&got) == norm(&c.expect) {
                    hits += 1;
                } else {
                    names.push(c.name);
                }
            }
            println!(
                "{:5} L={:3} rpl={:3}: {}/{} misses: {:?}",
                op,
                l,
                r,
                hits,
                cases.len(),
                names
            );
        }
    }
    // detailed dump for the best few configs
    if std::env::var("DUMP").is_ok() {
        for c in &cases {
            println!("=== {}", c.name);
            for (op, l, r) in [("fcat", 130, 1.5), ("fsep", 130, 1.5)] {
                let got = render_split(op, l, r, &c.cells);
                println!("  {op} L={l} rpl={r}: {:?}", got);
            }
            println!("  expect: {:?}", c.expect);
        }
    }
}

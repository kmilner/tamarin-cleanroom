//! Probe helper: read a flat cell text from argv[1]; print, for each candidate
//! line width W, the exact-engine record-label bytes. Used by the round-8 live
//! probe analysis to back out the effective per-cell width HughesPJ uses.
//!
//! With env LINELEN + RIBBON set, render the cell at a FIXED lineLength/ribbon
//! (to probe the ribbon-driven ragged fill) instead of sweeping width.
use graph_clean::doclayout::{wrap_cell_dot, wrap_cell_dot_lr};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let flat = &args[1];
    if let (Ok(ll), Ok(rib)) = (std::env::var("LINELEN"), std::env::var("RIBBON")) {
        let ll: isize = ll.parse().unwrap();
        let rib: f64 = rib.parse().unwrap();
        println!("{}", wrap_cell_dot_lr(flat, ll, rib));
        return;
    }
    let lo: isize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
    let hi: isize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(90);
    for w in lo..=hi {
        println!("{}\t{}", w, wrap_cell_dot(flat, w));
    }
}

#!/usr/bin/env python3
"""Test b_i = 87 - mw_j (sibling observed max line width) on 2-cell rows.

Uses bands.tsv for the b_i bands and re-reads the corpus for observed mw.
Prints hit rates of several residual identities against the bands.
"""
import sys, os, math
from collections import Counter
from fit import parse
from rowstats import parse_file, is_info, cell_lines, flat_width

def rib(L):
    v = L / 1.5
    fl = math.floor(v)
    if abs(v - fl - 0.5) < 1e-9:
        return fl if fl % 2 == 0 else fl + 1
    return round(v)

def main():
    bands_path, corpus = sys.argv[1], sys.argv[2]
    groups = parse(bands_path)
    # index bands by (file, kind, tuple(flats))
    bidx = {}
    for g in groups:
        flats = tuple(c[0] for c in g["cells"])
        bidx[(g["file"], g["kind"], flats)] = g
    stats = Counter()
    ex = []
    files = sorted(set(g["file"] for g in groups if len(g["cells"]) == 2))
    step = max(1, len(files) // 800)
    for fi, fn in enumerate(files):
        if fi % step:
            continue
        path = os.path.join(corpus, fn)
        if not os.path.exists(path):
            continue
        for recs in parse_file(path):
            info_idx = None
            for gi, cells in enumerate(recs):
                if len(cells) == 1 and is_info(cells[0]):
                    info_idx = gi
            if info_idx is None:
                continue
            for gi, cells in enumerate(recs):
                if gi == info_idx or len(cells) != 2 or any(c is None for c in cells):
                    continue
                if any(is_info(c) for c in cells):
                    continue
                if not any("\\l" in c for c in cells):
                    continue
                kind = "P" if gi < info_idx else "C"
                flats = tuple(flat_width(c) for c in cells)
                g = bidx.get((fn, kind, flats))
                if g is None:
                    continue
                mws = [max(cell_lines(c)) for c in cells]
                hs = [len(cell_lines(c)) for c in cells]
                for i in (0, 1):
                    c = g["cells"][i]
                    j = 1 - i
                    if c[1] != "wrap" or c[2] is None:
                        continue
                    lo, hi = c[2][0][0], c[2][-1][1]
                    rlo, rhi = rib(lo), rib(hi)
                    stats["cells"] += 1
                    # candidate identities, ribbon space
                    for name, val in (
                        ("87-mw_sib", 87 - mws[j]),
                        ("87-mw_sib,fl20", max(87 - mws[j], 20)),
                        ("87-flat_sib,fl20", max(87 - flats[j], 20)),
                        ("prop87", max(round(87 * flats[i] / sum(flats)), 20)),
                        ("mw_self", mws[i]),
                    ):
                        if rlo <= val <= rhi:
                            stats[name] += 1
                    if len(ex) < 25 and not (rlo <= max(87 - mws[j], 20) <= rhi):
                        ex.append((fn, kind, flats, i, (rlo, rhi), mws, hs))
    print("2-cell wrapping cells tested:", stats["cells"])
    for k in ("87-mw_sib", "87-mw_sib,fl20", "87-flat_sib,fl20", "prop87", "mw_self"):
        print(f"  {k:20s}: {stats[k]} = {100*stats[k]/max(stats['cells'],1):.1f}%")
    print("\nmisses of 87-mw_sib,fl20 (file kind flats i band mws heights):")
    for e in ex:
        print("  ", e)

if __name__ == "__main__":
    main()

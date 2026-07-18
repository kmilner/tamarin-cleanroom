#!/usr/bin/env python3
"""Dump abbr-free n>=3 rows with wrapping cells: flats, shapes, observed ribbon
bands, shipped predictions, hit/miss -- to eyeball the allocation structure."""
import sys, math, collections
from eval import parse, band_runs, in_band
from seqfit import C_of, N_of, prop, clamp

def l2r(L):
    v = L / 1.5
    fl = math.floor(v)
    if abs(v - fl - 0.5) < 1e-9:
        f = int(fl)
        return f if f % 2 == 0 else f + 1
    return round(v)

def rr(st):
    if st.startswith("F"):
        return st
    rs = band_runs(st)
    if not rs:
        return "NONE"
    return ",".join(f"{l2r(a)}..{l2r(b)}" for a, b in rs)

def main():
    rows = parse(sys.argv[1] if len(sys.argv) > 1 else "bands6.tsv")
    n_want = int(sys.argv[2]) if len(sys.argv) > 2 else 3
    shown = 0
    seen = set()
    for fname, kind, cells in rows:
        if len(cells) != n_want or any(c["nabbr"] for c in cells):
            continue
        wraps = [c["wrap"] for c in cells]
        if sum(wraps) < 2:
            continue
        C = C_of(cells)
        preds = [prop(cells, i, C) if wraps[i] else None for i in range(len(cells))]
        misses = [i for i in range(len(cells))
                  if wraps[i] and not in_band(cells[i]["st"], 3 * preds[i] // 2)]
        if not misses:
            continue
        key = tuple((c["f"], c["st"]) for c in cells)
        if key in seen:
            continue
        seen.add(key)
        desc = []
        for i, c in enumerate(cells):
            tag = "W" if wraps[i] else "flat"
            hit = "" if i not in misses else " MISS"
            pb = preds[i] if preds[i] is not None else "-"
            desc.append(f"[f={c['f']} C={C[i]} N={N_of(c)} {tag} band={rr(c['st'])} pred={pb}{hit}]")
        print(fname, kind if False else "", " ".join(desc))
        shown += 1
        if shown >= 40:
            break

if __name__ == "__main__":
    main()

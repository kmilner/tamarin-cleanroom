#!/usr/bin/env python3
"""Generalized n-cell fill-allocation fitting on abbr-free rows (all cells
nabbr==0) with at least one wrapping cell.  Score: per wrapping cell, predicted
L=3b//2 in band; also per-row all-hit."""
import sys, math, collections
from eval import parse, band_runs, in_band

def clamp(b, f):
    return max(20, min(b, max(f - 1, 20)))

def C_of(cells):
    return [c["f"] + c["tup"] + c["uni"] for c in cells]

def N_of(c):
    return c["f"] + c["uni"] + c["nfunc"]

def prop(cells, i, C, charges=None):
    """Shipped proportional fill for cell i; sibling charge defaults to C_j."""
    c = cells[i]
    num = float(N_of(c))
    t = num
    for j, cj in enumerate(cells):
        if j != i:
            ch = C[j] if charges is None else charges[j]
            w = 5.0/6.0 if (cj["sqa"] and c["tup"] > 0) else 1.0
            t += w * ch
    return clamp(math.floor(87.0 * num / t + 0.5), c["f"])

def m_shipped(cells, wraps):
    C = C_of(cells)
    return [prop(cells, i, C) if wraps[i] else None for i in range(len(cells))]

def m_seq_desc(cells, wraps):
    """Widest-first sequential: widest wrapping cell gets prop (sibs at C);
    each subsequent wrapping cell gets 87 - sum(processed wrapping allocs)
    - sum(other flats' C), floored/clamped."""
    n = len(cells)
    C = C_of(cells)
    order = sorted((i for i in range(n) if wraps[i]), key=lambda i: -cells[i]["f"])
    out = [None] * n
    for k, i in enumerate(order):
        if k == 0:
            out[i] = prop(cells, i, C)
        else:
            rem = 87
            for j in range(n):
                if j == i:
                    continue
                if wraps[j] and out[j] is not None:
                    rem -= out[j]
                else:
                    rem -= C[j]
            out[i] = clamp(rem, cells[i]["f"])
    return out

def m_seq_desc_all_prop_first(cells, wraps):
    """Every wrapping cell's sibling-charge = prop_j if sibling wraps else C_j
    (one refinement pass, symmetric) -- except the WIDEST wrapping cell, which
    keeps plain C charges."""
    n = len(cells)
    C = C_of(cells)
    p = [prop(cells, i, C) if wraps[i] else None for i in range(n)]
    widest = max((i for i in range(n) if wraps[i]), key=lambda i: cells[i]["f"], default=None)
    out = [None] * n
    for i in range(n):
        if not wraps[i]:
            continue
        if i == widest:
            out[i] = p[i]
        else:
            charges = [(p[j] if (wraps[j] and p[j] is not None) else C[j]) for j in range(n)]
            rem = 87 - sum(charges[j] for j in range(n) if j != i)
            out[i] = clamp(rem, cells[i]["f"])
    return out

def m_seq_charges_prop(cells, wraps):
    """Proportional but wrapping siblings charged at their prop value (one
    pass), fitting siblings at C."""
    n = len(cells)
    C = C_of(cells)
    p = [prop(cells, i, C) if wraps[i] else None for i in range(n)]
    charges = [(p[j] if wraps[j] else C[j]) for j in range(n)]
    return [prop(cells, i, C, charges) if wraps[i] else None for i in range(n)]

MODELS = {
    "shipped": m_shipped,
    "seq_desc": m_seq_desc,
    "sym_refine": m_seq_desc_all_prop_first,
    "prop_charges": m_seq_charges_prop,
}

def main():
    rows = parse(sys.argv[1] if len(sys.argv) > 1 else "bands6.tsv")
    pool = []
    for fname, kind, cells in rows:
        if any(c["nabbr"] for c in cells):
            continue
        wraps = [c["wrap"] for c in cells]
        if len(cells) >= 2 and any(wraps):
            pool.append((fname, cells, wraps))
    by_n = collections.Counter(len(c) for _, c, _ in pool)
    print("abbr-free multi rows with a wrap, by n:", dict(sorted(by_n.items())))
    for name, fn in MODELS.items():
        stat = collections.defaultdict(lambda: [0, 0, 0, 0])  # n,nwrap -> hitcell,totcell,hitrow,totrow
        for fname, cells, wraps in pool:
            bs = fn(cells, wraps)
            oks = []
            for c, b in zip(cells, bs):
                if b is not None:
                    oks.append(in_band(c["st"], 3 * b // 2))
            key = (len(cells), sum(wraps))
            s = stat[key]
            s[0] += sum(oks); s[1] += len(oks); s[2] += all(oks); s[3] += 1
        tot = [0, 0, 0, 0]
        for s in stat.values():
            for k in range(4):
                tot[k] += s[k]
        print(f"\n{name}: cells {tot[0]}/{tot[1]} = {100*tot[0]/tot[1]:.2f}%  rows {tot[2]}/{tot[3]}")
        for key in sorted(stat):
            s = stat[key]
            print(f"   n={key[0]} wraps={key[1]}: cells {s[0]}/{s[1]}  rows {s[2]}/{s[3]}")

if __name__ == "__main__":
    main()

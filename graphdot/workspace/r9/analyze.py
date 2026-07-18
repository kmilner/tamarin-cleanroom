#!/usr/bin/env python3
"""Analyze prop(87) misses against per-cell L-bands; look for structure."""
import sys, math
from collections import Counter, defaultdict
from fit import parse, rib_to_L, ok, alloc_prop

groups = parse(sys.argv[1] if len(sys.argv) > 1 else "bands.tsv")
multi = [g for g in groups if len(g["cells"]) >= 2]

# ribbon-space band: for L in band runs, ribbon = banker's round(L/1.5)
def rib(L):
    v = L / 1.5
    fl = math.floor(v)
    if abs(v - fl - 0.5) < 1e-9:
        return fl if fl % 2 == 0 else fl + 1
    return round(v)

miss_dir = Counter()
by_n = defaultdict(lambda: [0, 0])
by_kind = defaultdict(lambda: [0, 0])
deltas = Counter()
miss_examples = []
# Correlate miss direction with composition features
feat = defaultdict(lambda: [0, 0])

for g in multi:
    flats = [c[0] for c in g["cells"]]
    T = sum(flats)
    n = len(flats)
    budgets = alloc_prop(flats)
    Ls = [rib_to_L(b) for b in budgets]
    nwrapping = sum(1 for c in g["cells"] if c[1] == "wrap")
    for i, (c, L, b) in enumerate(zip(g["cells"], Ls, budgets)):
        if c[1] != "wrap" or c[2] is None:
            continue
        hit = ok(c, L)
        by_n[n][0 if hit else 1] += 1
        by_kind[g["kind"]][0 if hit else 1] += 1
        if not hit:
            lo, hi = c[2][0][0], c[2][-1][1]
            # signed distance in ribbon space
            rlo, rhi = rib(lo), rib(hi)
            if L < lo:
                d = rlo - b
                miss_dir["alloc-too-small"] += 1
            elif L > hi:
                d = rhi - b
                miss_dir["alloc-too-big"] += 1
            else:
                d = 0
                miss_dir["in-gap-between-runs"] += 1
            deltas[max(-30, min(30, d))] += 1
            # features
            other = T - flats[i]
            feat[("wrapsibs>0", any(g["cells"][j][1] == "wrap" for j in range(n) if j != i))][0 if hit else 1] += 1
            if len(miss_examples) < 40 and d != 0:
                miss_examples.append((g["file"], g["kind"], flats, i, b, (rlo, rhi), d, g["info"]))
        else:
            feat[("wrapsibs>0", any(g["cells"][j][1] == "wrap" for j in range(n) if j != i))][0] += 1

print("miss directions:", dict(miss_dir))
print("by n cells (hit, miss):")
for n in sorted(by_n):
    h, m = by_n[n]
    print(f"  n={n}: {h}/{h+m} = {100*h/(h+m):.1f}%")
print("by group kind:", {k: (v[0], v[1]) for k, v in by_kind.items()})
print("delta (needed_ribbon - allocated_ribbon) histogram (capped +-30):")
for d in sorted(deltas):
    print(f"  {d:+3d}: {deltas[d]}")
print("\nmiss examples (file kind flats idx alloc_b band_rib delta info):")
for e in miss_examples[:30]:
    print("  ", e)

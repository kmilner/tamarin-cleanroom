#!/usr/bin/env python3
"""Regress pinned (narrow-band) cell budgets against group-composition features.

A pinned cell: wrapping, band width in ribbon space <= MAXW. Its observed budget
b_obs ~ band midpoint. Features: f, T, n, sum_others, sum_big_others,
n_small (flats<=20 among others), kind, info.
Least squares by hand (no numpy): solve normal equations via Gaussian elim.
"""
import sys, math
from collections import defaultdict
from fit import parse

def rib(L):
    v = L / 1.5
    fl = math.floor(v)
    if abs(v - fl - 0.5) < 1e-9:
        return fl if fl % 2 == 0 else fl + 1
    return round(v)

def gather(groups, maxw=2):
    rows = []
    for g in groups:
        cells = g["cells"]
        if len(cells) < 2:
            continue
        flats = [c[0] for c in cells]
        T = sum(flats)
        n = len(flats)
        for i, c in enumerate(cells):
            if c[1] != "wrap" or c[2] is None:
                continue
            lo, hi = c[2][0][0], c[2][-1][1]
            rlo, rhi = rib(lo), rib(hi)
            if rhi - rlo > maxw:
                continue
            others = [flats[j] for j in range(n) if j != i]
            rows.append({
                "f": flats[i], "T": T, "n": n,
                "so": sum(others),
                "sbig": sum(x for x in others if x > 20),
                "nsmall": sum(1 for x in others if x <= 20),
                "nwrapo": sum(1 for j in range(n) if j != i and cells[j][1] == "wrap"),
                "b": (rlo + rhi) / 2, "rlo": rlo, "rhi": rhi,
                "kind": g["kind"], "info": g["info"], "file": g["file"],
                "flats": flats, "i": i,
            })
    return rows

def lstsq(X, y):
    k = len(X[0])
    A = [[sum(X[r][a] * X[r][b] for r in range(len(X))) for b in range(k)] for a in range(k)]
    v = [sum(X[r][a] * y[r] for r in range(len(X))) for a in range(k)]
    for c in range(k):
        p = max(range(c, k), key=lambda r: abs(A[r][c]))
        A[c], A[p] = A[p], A[c]
        v[c], v[p] = v[p], v[c]
        for r in range(c + 1, k):
            m = A[r][c] / A[c][c]
            for cc in range(c, k):
                A[r][cc] -= m * A[c][cc]
            v[r] -= m * v[c]
    beta = [0.0] * k
    for r in range(k - 1, -1, -1):
        beta[r] = (v[r] - sum(A[r][c] * beta[c] for c in range(r + 1, k))) / A[r][r]
    return beta

if __name__ == "__main__":
    groups = parse(sys.argv[1] if len(sys.argv) > 1 else "bands.tsv")
    rows = gather(groups, maxw=2)
    # dedupe identical (flats, i, band) observations to avoid corpus duplication bias
    seen = {}
    for r in rows:
        key = (tuple(r["flats"]), r["i"], r["rlo"], r["rhi"])
        seen.setdefault(key, r)
    rows = list(seen.values())
    print(f"pinned unique cells: {len(rows)}")
    # global linear fit b ~ a*f + b*sbig + c*nsmall + d*n + e
    X = [[r["f"], r["sbig"], r["nsmall"], r["n"], 1.0] for r in rows]
    y = [r["b"] for r in rows]
    beta = lstsq(X, y)
    pred = [sum(x * b for x, b in zip(row, beta)) for row in X]
    inband = sum(1 for r, p in zip(rows, pred) if r["rlo"] <= p <= r["rhi"])
    resid = [p - yy for p, yy in zip(pred, y)]
    rmse = math.sqrt(sum(e * e for e in resid) / len(resid))
    print("beta [f, sbig, nsmall, n, 1] =", [round(b, 4) for b in beta])
    print(f"rmse {rmse:.2f}; in-band {inband}/{len(rows)}")
    # fit separately by n
    byn = defaultdict(list)
    for r in rows:
        byn[r["n"]].append(r)
    for n in sorted(byn):
        rs = byn[n]
        if len(rs) < 12:
            continue
        X = [[r["f"], r["sbig"], r["nsmall"], 1.0] for r in rs]
        y = [r["b"] for r in rs]
        beta = lstsq(X, y)
        pred = [sum(x * b for x, b in zip(row, beta)) for row in X]
        inb = sum(1 for r, p in zip(rs, pred) if r["rlo"] <= p <= r["rhi"])
        rmse = math.sqrt(sum((p - yy) ** 2 for p, yy in zip(pred, y)) / len(rs))
        print(f"n={n}: {len(rs)} cells, beta[f,sbig,nsmall,1]={[round(b,3) for b in beta]}, rmse {rmse:.2f}, in-band {inb}/{len(rs)}")
    # dump n=2 pinned pairs for eyeballing
    print("\nn=2 pinned cells (f, sibling, b rlo-rhi):")
    n2 = sorted([r for r in rows if r["n"] == 2], key=lambda r: (r["f"], r["so"]))
    for r in n2[:80]:
        print(f"  f={r['f']:4d} sib={r['so']:4d} b={r['rlo']}-{r['rhi']} file={r['file']} i={r['i']}")

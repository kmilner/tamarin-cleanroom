#!/usr/bin/env python3
"""Fit the per-group width allocation against corpus-derived L-space bands.

Input: bands.tsv from band_dump (file kind info_flat prem_flats concl_flats cells)
Each cell entry: flat:STATUS, STATUS in {F<lo>, FNONE, NONE, lo-hi[,lo-hi...]}.
Constraint semantics for an allocator that assigns line length L_i to cell i:
  F<lo>  -> ok iff L_i >= lo
  runs   -> ok iff L_i inside a run
  NONE   -> never ok (cell-doc ceiling exclusion; tracked separately)
"""
import sys
from collections import Counter, defaultdict

def parse(path):
    groups = []
    with open(path) as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if len(parts) != 6:
                continue
            fname, kind, info_flat, pf, cf, cells = parts
            cellrecs = []
            for c in cells.split("|"):
                flat_s, status = c.split(":", 1)
                flat = int(flat_s)
                if status.startswith("F"):
                    lo = None if status == "FNONE" else int(status[1:])
                    cellrecs.append((flat, "fit", lo))
                elif status == "NONE":
                    cellrecs.append((flat, "wrap", None))
                else:
                    runs = []
                    for r in status.split(","):
                        a, b = r.split("-")
                        runs.append((int(a), int(b)))
                    cellrecs.append((flat, "wrap", runs))
            groups.append({
                "file": fname, "kind": kind, "info": int(info_flat),
                "pf": [int(x) for x in pf.split(",")] if pf else [],
                "cf": [int(x) for x in cf.split(",")] if cf else [],
                "cells": cellrecs,
            })
    return groups

def rib_to_L(b):
    return (3 * b) // 2

def ok(cell, L):
    flat, k, x = cell
    if k == "fit":
        return x is not None and L >= x
    if x is None:
        return False
    return any(a <= L <= b for a, b in x)

# ---------------- allocator families (ribbon space unless noted) -------------

def rnd_half(x):  # round half away from zero (matches (W*f + t/2)//t for ints)
    import math
    return math.floor(x + 0.5)

def alloc_prop(flats, W=87, floor=20, rnd=rnd_half):
    t = sum(flats)
    if t == 0:
        return [W] * len(flats)
    return [max(rnd(W * f / t), floor) for f in flats]

def alloc_prop_floor_r(flats, W=87, floor=20):
    import math
    t = sum(flats)
    return [max(math.floor(W * f / t), floor) for f in flats]

def alloc_prop_ceil_r(flats, W=87, floor=20):
    import math
    t = sum(flats)
    return [max(math.ceil(W * f / t), floor) for f in flats]

def alloc_prop_redist(flats, W=87, floor=20):
    """Clamp-at-floor then renormalize the rest (iterate to fixpoint)."""
    n = len(flats)
    fixed = {}
    while True:
        rest = [i for i in range(n) if i not in fixed]
        if not rest:
            break
        t = sum(flats[i] for i in rest)
        w = W - sum(fixed.values())
        if t == 0 or w <= 0:
            for i in rest:
                fixed[i] = floor
            break
        newly = [i for i in rest if w * flats[i] / t < floor]
        if not newly:
            for i in rest:
                fixed[i] = max(rnd_half(w * flats[i] / t), floor)
            break
        for i in newly:
            fixed[i] = floor
    return [fixed[i] for i in range(n)]

def alloc_fitfix(flats, W=87, floor=20):
    """Cells whose share covers their flat are fixed at flat; redistribute."""
    n = len(flats)
    fixed = {}
    while True:
        rest = [i for i in range(n) if i not in fixed]
        if not rest:
            break
        t = sum(flats[i] for i in rest)
        w = W - sum(fixed.values())
        if t == 0 or w <= 0:
            for i in rest:
                fixed[i] = floor
            break
        newly = [i for i in rest if max(rnd_half(w * flats[i] / t), floor) >= flats[i]]
        if not newly:
            for i in rest:
                fixed[i] = max(rnd_half(w * flats[i] / t), floor)
            break
        for i in newly:
            fixed[i] = flats[i]
    return [fixed[i] for i in range(n)]

def eval_alloc(groups, fn, space="ribbon", ctx=False):
    """Returns (band_hits, band_total, fit_viol, fit_total, none_count)."""
    hits = tot = fviol = ftot = nones = 0
    for g in groups:
        flats = [c[0] for c in g["cells"]]
        if len(flats) < 2:
            continue
        if ctx:
            Ls = fn(flats, g)
        else:
            Ls = fn(flats)
        if space == "ribbon":
            Ls = [rib_to_L(b) for b in Ls]
        for c, L in zip(g["cells"], Ls):
            if c[1] == "wrap":
                if c[2] is None:
                    nones += 1
                    continue
                tot += 1
                if ok(c, L):
                    hits += 1
            else:
                ftot += 1
                if not ok(c, L):
                    fviol += 1
    return hits, tot, fviol, ftot, nones

def report(name, r):
    hits, tot, fviol, ftot, nones = r
    allwrap = tot + nones
    print(f"{name:34s} band-hit {hits:>6}/{tot:<6}={100*hits/max(tot,1):6.2f}%  "
          f"incl-NONE {100*hits/max(allwrap,1):6.2f}%  "
          f"fit-viol {fviol:>5}/{ftot:<6}={100*fviol/max(ftot,1):5.2f}%")

if __name__ == "__main__":
    groups = parse(sys.argv[1] if len(sys.argv) > 1 else "bands.tsv")
    multi = [g for g in groups if len(g["cells"]) >= 2]
    single = [g for g in groups if len(g["cells"]) == 1]
    wrap_multi = sum(1 for g in multi for c in g["cells"] if c[1] == "wrap")
    none_multi = sum(1 for g in multi for c in g["cells"] if c[1] == "wrap" and c[2] is None)
    wrap_single = sum(1 for g in single for c in g["cells"] if c[1] == "wrap")
    none_single = sum(1 for g in single for c in g["cells"] if c[1] == "wrap" and c[2] is None)
    print(f"groups: {len(groups)} (multi {len(multi)}, single-dumped {len(single)})")
    print(f"multi wrapping cells: {wrap_multi}, band-NONE {none_multi} "
          f"(ceiling {100*(wrap_multi-none_multi)/max(wrap_multi,1):.2f}%)")
    print(f"single wrapping cells: {wrap_single}, band-NONE {none_single} "
          f"(ceiling {100*(wrap_single-none_single)/max(wrap_single,1):.2f}%)")

    report("prop(87,f20,half)", eval_alloc(multi, lambda f: alloc_prop(f)))
    report("prop(87,f20,floor)", eval_alloc(multi, lambda f: alloc_prop_floor_r(f)))
    report("prop(87,f20,ceil)", eval_alloc(multi, lambda f: alloc_prop_ceil_r(f)))
    report("prop_redist(87,f20)", eval_alloc(multi, lambda f: alloc_prop_redist(f)))
    report("fitfix(87,f20)", eval_alloc(multi, lambda f: alloc_fitfix(f)))
    for W in (85, 86, 88, 89, 90):
        report(f"prop({W},f20,half)", eval_alloc(multi, lambda f, W=W: alloc_prop(f, W=W)))
    for fl in (15, 18, 22, 25):
        report(f"prop(87,f{fl},half)", eval_alloc(multi, lambda f, fl=fl: alloc_prop(f, floor=fl)))

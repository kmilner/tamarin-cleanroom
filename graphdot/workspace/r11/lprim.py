#!/usr/bin/env python3
"""Score the L-primitive fill model against probe + corpus band TSVs.

Model: trigger as shipped (C = f+tup+uni, bonus, floor 20, lone 87).
Fill:  t = N + sum(w_j * C_j);  L = floor(130.5 * N / t)  clamped to
       [30, fitL(flat)-1]; hit iff L in the observed band.
Compare against the shipped ribbon model (half-up) and ribbon half-down."""
import math, sys, glob
sys.path.insert(0, '.')
from eval import band_runs, in_band

def parse_any(path):
    rows = []
    for line in open(path):
        p = line.rstrip("\n").split("\t")
        if len(p) != 6:
            continue
        cells = []
        ok = True
        for cc in p[5].split("|"):
            seg = cc.split(":")
            if len(seg) not in (11, 14):
                ok = False
                break
            f = int(seg[0]); st = seg[1]
            vals = list(map(int, seg[2:]))
            if len(seg) == 11:
                dtop, drec, nq, sqa, nargs, nfunc, nabbr, ctup, bmax = vals
                tup, uni, cbmax = ctup, 0, (bmax if bmax <= 6 else 4)
            else:
                (dtop, drec, nq, sqa, nargs, nfunc, nabbr, ctup, bmax,
                 tup, uni, cbmax) = vals
            cells.append(dict(f=f, st=st, wrap=not st.startswith("F"), sqa=sqa,
                              nfunc=nfunc, nabbr=nabbr, tup=tup, uni=uni, bonus=cbmax))
        if ok and cells:
            rows.append((p[0], cells))
    return rows

def fit_l(flat):
    l = math.floor(flat * 1.5) - 2
    if l < 8:
        l = 8
    while True:
        v = l / 1.5
        fl = math.floor(v)
        if abs(v - fl - 0.5) < 1e-9:
            r = int(fl) if int(fl) % 2 == 0 else int(fl) + 1
        else:
            r = round(v)
        if r >= flat:
            return l
        l += 1

def score(rows, mode):
    hit = tot = trigerr = 0
    misses = []
    for fname, cells in rows:
        n = len(cells)
        C = [c["f"] + c["tup"] + c["uni"] for c in cells]
        S = sum(C)
        for i, c in enumerate(cells):
            f = c["f"]
            if n == 1:
                b_t = 87
            else:
                b_t = max(87 + c["bonus"] - (S - C[i]), 20)
            wraps = f > b_t
            if wraps != c["wrap"]:
                trigerr += 1
                continue
            if not wraps:
                continue
            tot += 1
            num = float(f + c["uni"] + c["nfunc"])
            t = num
            for j, cj in enumerate(cells):
                if j != i:
                    w = 5.0 / 6.0 if (cj["sqa"] and c["tup"] > 0) else 1.0
                    t += w * C[j]
            if n == 1:
                L = 130
            elif mode == "L":
                L = math.floor(130.5 * num / t)
                L = max(30, min(L, fit_l(f) - 1))
            else:
                x = 87.0 * num / t
                if mode == "hd":
                    fl = math.floor(x)
                    b = int(fl) if (x - fl) == 0.5 else int(math.floor(x + 0.5))
                else:
                    b = int(math.floor(x + 0.5))
                b = max(20, min(b, max(f - 1, 20)))
                L = 3 * b // 2
            if in_band(c["st"], L):
                hit += 1
            else:
                misses.append((fname, f, [cc["f"] for cc in cells], L, c["st"]))
    return hit, tot, trigerr, misses

def main():
    groups = [("GHI", ["bandsGHI.tsv"]),
              ("r10 archived", sorted(glob.glob("../r10/b4_*.tsv")))]
    for name, paths in groups:
        rows = []
        for p in paths:
            rows.extend(parse_any(p))
        for mode in ("hu", "hd", "L"):
            h, t, te, m = score(rows, mode)
            print(f"{name:14s} {mode:2s}: fill {h}/{t}  trigerr {te}")
        if "-v" in sys.argv:
            for mm in score(rows, "L")[3]:
                print("   L-miss:", mm)

if __name__ == "__main__":
    main()

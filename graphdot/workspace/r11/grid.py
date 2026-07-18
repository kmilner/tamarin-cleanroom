#!/usr/bin/env python3
"""Grid search the fill law: L-vs-ribbon primitive x rounding x wrapping-sib
surcharge delta.  Trigger fixed as shipped.  Wrap-status for the charge comes
from the TRIGGER prediction (self-contained, no oracle peeking).
Scores: GHI probes, r10 archived probes, corpus bands6 (abbr-free cells and
all cells)."""
import math, sys, glob
sys.path.insert(0, '.')
from eval import in_band
from lprim import parse_any, fit_l

def score(rows, prim, rnd, delta, only_plain=False):
    hit = tot = 0
    for fname, cells in rows:
        n = len(cells)
        C = [c["f"] + c["tup"] + c["uni"] for c in cells]
        S = sum(C)
        # trigger-predicted wrap statuses
        pw = []
        for i, c in enumerate(cells):
            b_t = 87 if n == 1 else max(87 + c["bonus"] - (S - C[i]), 20)
            pw.append(c["f"] > b_t)
        for i, c in enumerate(cells):
            f = c["f"]
            if not (pw[i] and c["wrap"]):
                continue
            if only_plain and any(cc["nabbr"] for cc in cells):
                continue
            tot += 1
            num = float(f + c["uni"] + c["nfunc"])
            t = num
            for j, cj in enumerate(cells):
                if j != i:
                    w = 5.0 / 6.0 if (cj["sqa"] and c["tup"] > 0) else 1.0
                    t += w * (C[j] + (delta if pw[j] else 0))
            if n == 1:
                L = 130
            else:
                x = (130.5 if prim == "L" else 87.0) * num / t
                if rnd == "f":
                    v = math.floor(x)
                elif rnd == "n":
                    v = math.floor(x + 0.5)
                else:  # hd
                    fl = math.floor(x)
                    v = int(fl) if (x - fl) == 0.5 else int(math.floor(x + 0.5))
                if prim == "L":
                    L = max(30, min(v, fit_l(f) - 1))
                else:
                    b = max(20, min(v, max(f - 1, 20)))
                    L = 3 * b // 2
            if in_band(c["st"], L):
                hit += 1
    return hit, tot

def main():
    ghi = parse_any("bandsGHI.tsv")
    arch = []
    for p in sorted(glob.glob("../r10/b4_*.tsv")):
        arch.extend(parse_any(p))
    corpus = parse_any("bands6.tsv")
    print(f"{'model':22s} {'GHI':>9s} {'archived':>9s} {'corpus-plain':>12s} {'corpus-all':>10s}")
    for prim in ("r", "L"):
        for rnd in ("f", "n", "hd"):
            for delta in (0, 1, 2, 3):
                g = score(ghi, prim, rnd, delta)
                a = score(arch, prim, rnd, delta)
                cp = score(corpus, prim, rnd, delta, only_plain=True)
                ca = score(corpus, prim, rnd, delta)
                print(f"{prim}/{rnd}/d{delta:<18d} {g[0]:>4d}/{g[1]:<4d} {a[0]:>4d}/{a[1]:<4d} "
                      f"{cp[0]:>6d}/{cp[1]:<6d} {ca[0]:>6d}/{ca[1]:<6d}")

if __name__ == "__main__":
    main()
